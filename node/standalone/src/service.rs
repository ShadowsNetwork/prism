// Copyright 2019-2020 ShadowsNetwork Inc.
// This file is part of Shadows.

// Shadows is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Shadows is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Shadows.  If not, see <http://www.gnu.org/licenses/>.

//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use crate::mock_timestamp::MockTimestampInherentDataProvider;
use fc_consensus::FrontierBlockImport;
use fc_rpc_core::types::PendingTransactions;
use shadows_runtime::{self, opaque::Block, RuntimeApi};
use parity_scale_codec::Encode;
use sc_client_api::{BlockchainEvents, ExecutorProvider, RemoteBackend};
use sc_consensus_manual_seal::{self as manual_seal};
use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;
use sc_finality_grandpa::{GrandpaBlockImport, SharedVoterState};
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;
use sp_core::H160;
use sp_inherents::InherentDataProviders;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
	time::Duration,
};

// Our native executor instance.
native_executor_instance!(
	pub Executor,
	shadows_runtime::api::dispatch,
	shadows_runtime::native_version,
);

/// Build the inherent data providers (timestamp and authorship) for the node.
pub fn build_inherent_data_providers(
	manual_seal: bool,
	author: Option<H160>,
) -> Result<InherentDataProviders, sc_service::Error> {
	let providers = InherentDataProviders::new();
	if let Some(account) = author {
		providers
			.register_provider(author_inherent::InherentDataProvider(account.encode()))
			.map_err(Into::into)
			.map_err(sp_consensus::error::Error::InherentData)?;
	}
	if manual_seal {
		providers
			.register_provider(MockTimestampInherentDataProvider)
			.map_err(Into::into)
			.map_err(sp_consensus::error::Error::InherentData)?;
	} else {
		providers
			.register_provider(sp_timestamp::InherentDataProvider)
			.map_err(Into::into)
			.map_err(sp_consensus::error::Error::InherentData)?;
	}

	Ok(providers)
}

type FullClient = sc_service::TFullClient<Block, RuntimeApi, Executor>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

pub enum ConsensusResult {
	Aura(
		sc_consensus_aura::AuraBlockImport<
			Block,
			FullClient,
			FrontierBlockImport<
				Block,
				GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>,
				FullClient,
			>,
			AuraPair,
		>,
		sc_finality_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
	),
	ManualSeal(FrontierBlockImport<Block, Arc<FullClient>, FullClient>),
}

pub fn new_partial(
	config: &Configuration,
	manual_seal: bool,
	author: Option<H160>,
) -> Result<
	sc_service::PartialComponents<
		FullClient,
		FullBackend,
		FullSelectChain,
		sp_consensus::import_queue::BasicQueue<Block, sp_api::TransactionFor<FullClient, Block>>,
		sc_transaction_pool::FullPool<Block, FullClient>,
		(ConsensusResult, PendingTransactions),
	>,
	ServiceError,
> {
	let inherent_data_providers = build_inherent_data_providers(manual_seal, author)?;

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, Executor>(&config)?;
	let client = Arc::new(client);

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
	);

	let pending_transactions: PendingTransactions = Some(Arc::new(Mutex::new(HashMap::new())));

	if manual_seal {
		let frontier_block_import = FrontierBlockImport::new(client.clone(), client.clone(), true);

		let import_queue = sc_consensus_manual_seal::import_queue(
			Box::new(frontier_block_import.clone()),
			&task_manager.spawn_handle(),
			config.prometheus_registry(),
		);

		return Ok(sc_service::PartialComponents {
			client,
			backend,
			task_manager,
			import_queue,
			keystore_container,
			select_chain,
			transaction_pool,
			inherent_data_providers,
			other: (
				ConsensusResult::ManualSeal(frontier_block_import),
				pending_transactions,
			),
		});
	}

	let (grandpa_block_import, grandpa_link) = sc_finality_grandpa::block_import(
		client.clone(),
		&(client.clone() as Arc<_>),
		select_chain.clone(),
	)?;

	let frontier_block_import =
		FrontierBlockImport::new(grandpa_block_import.clone(), client.clone(), true);

	let aura_block_import = sc_consensus_aura::AuraBlockImport::<_, _, _, AuraPair>::new(
		frontier_block_import,
		client.clone(),
	);

	let import_queue = sc_consensus_aura::import_queue::<_, _, _, AuraPair, _, _>(
		sc_consensus_aura::slot_duration(&*client)?,
		aura_block_import.clone(),
		Some(Box::new(grandpa_block_import.clone())),
		client.clone(),
		inherent_data_providers.clone(),
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
		sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone()),
	)?;

	Ok(sc_service::PartialComponents {
		client,
		backend,
		task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		inherent_data_providers,
		other: (
			ConsensusResult::Aura(aura_block_import, grandpa_link),
			pending_transactions,
		),
	})
}

/// Builds a new service for a full client.
pub fn new_full(
	config: Configuration,
	manual_seal: bool,
	author_id: Option<H160>,
) -> Result<TaskManager, ServiceError> {
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		inherent_data_providers,
		other: (consensus_result, pending_transactions),
	} = new_partial(&config, manual_seal, author_id)?;

	let (network, network_status_sinks, system_rpc_tx, network_starter) = match consensus_result {
		ConsensusResult::ManualSeal(_) => {
			sc_service::build_network(sc_service::BuildNetworkParams {
				config: &config,
				client: client.clone(),
				transaction_pool: transaction_pool.clone(),
				spawn_handle: task_manager.spawn_handle(),
				import_queue,
				on_demand: None,
				block_announce_validator_builder: None,
			})?
		}
		ConsensusResult::Aura(_, _) => sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: None,
			block_announce_validator_builder: None,
		})?,
	};

	// Channel for the rpc handler to communicate with the authorship task.
	let (command_sink, commands_stream) = futures::channel::mpsc::channel(1000);

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			backend.clone(),
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let role = config.role.clone();
	let force_authoring = config.force_authoring;
	// Don't backoff authoring. See https://github.com/paritytech/substrate/pull/7186 for details
	let backoff_authoring_blocks: Option<()> = None;
	let name = config.network.node_name.clone();
	let enable_grandpa = !config.disable_grandpa;
	let prometheus_registry = config.prometheus_registry().cloned();
	let telemetry_connection_sinks = sc_service::TelemetryConnectionSinks::default();
	let is_authority = role.is_authority();
	let subscription_task_executor =
		sc_rpc::SubscriptionTaskExecutor::new(task_manager.spawn_handle());

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let network = network.clone();
		let pending = pending_transactions.clone();
		Box::new(move |deny_unsafe, _| {
			let deps = shadows_rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				graph: pool.pool().clone(),
				deny_unsafe,
				is_authority,
				network: network.clone(),
				pending_transactions: pending.clone(),
				command_sink: Some(command_sink.clone()),
			};
			shadows_rpc::create_full(deps, subscription_task_executor.clone())
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network: network.clone(),
		client: client.clone(),
		keystore: keystore_container.sync_keystore(),
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		telemetry_connection_sinks: telemetry_connection_sinks.clone(),
		rpc_extensions_builder: rpc_extensions_builder,
		on_demand: None,
		remote_blockchain: None,
		backend,
		network_status_sinks,
		system_rpc_tx,
		config,
	})?;

	// Spawn Frontier pending transactions maintenance task (as essential, otherwise we leak).
	if pending_transactions.is_some() {
		use fp_consensus::{ConsensusLog, FRONTIER_ENGINE_ID};
		use futures::StreamExt;
		use sp_runtime::generic::OpaqueDigestItemId;

		const TRANSACTION_RETAIN_THRESHOLD: u64 = 5;
		task_manager.spawn_essential_handle().spawn(
			"frontier-pending-transactions",
			client
				.import_notification_stream()
				.for_each(move |notification| {
					if let Ok(locked) = &mut pending_transactions.clone().unwrap().lock() {
						// As pending transactions have a finite lifespan anyway
						// we can ignore MultiplePostRuntimeLogs error checks.
						let mut frontier_log: Option<_> = None;
						for log in notification.header.digest.logs {
							let log = log.try_to::<ConsensusLog>(OpaqueDigestItemId::Consensus(
								&FRONTIER_ENGINE_ID,
							));
							if let Some(log) = log {
								frontier_log = Some(log);
							}
						}

						let imported_number: u64 = notification.header.number as u64;

						if let Some(ConsensusLog::EndBlock {
							block_hash: _,
							transaction_hashes,
						}) = frontier_log
						{
							// Retain all pending transactions that were not
							// processed in the current block.
							locked.retain(|&k, _| !transaction_hashes.contains(&k));
						}
						locked.retain(|_, v| {
							// Drop all the transactions that exceeded the given lifespan.
							let lifespan_limit = v.at_block + TRANSACTION_RETAIN_THRESHOLD;
							lifespan_limit > imported_number
						});
					}
					futures::future::ready(())
				}),
		);
	}

	match consensus_result {
		ConsensusResult::ManualSeal(block_import) => {
			if role.is_authority() {
				let env = sc_basic_authorship::ProposerFactory::new(
					task_manager.spawn_handle(),
					client.clone(),
					transaction_pool.clone(),
					prometheus_registry.as_ref(),
				);

				// Background authorship future
				let authorship_future =
					manual_seal::run_manual_seal(manual_seal::ManualSealParams {
						block_import,
						env,
						client,
						pool: transaction_pool.pool().clone(),
						commands_stream,
						select_chain,
						consensus_data_provider: None,
						inherent_data_providers,
					});

				// we spawn the future on a background thread managed by service.
				task_manager
					.spawn_essential_handle()
					.spawn_blocking("manual-seal", authorship_future);
			}
			log::info!("Manual Seal Ready");
		}
		ConsensusResult::Aura(aura_block_import, grandpa_link) => {
			if role.is_authority() {
				let proposer = sc_basic_authorship::ProposerFactory::new(
					task_manager.spawn_handle(),
					client.clone(),
					transaction_pool.clone(),
					prometheus_registry.as_ref(),
				);

				let can_author_with =
					sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());
				let aura = sc_consensus_aura::start_aura::<_, _, _, _, _, AuraPair, _, _, _, _>(
					sc_consensus_aura::slot_duration(&*client)?,
					client.clone(),
					select_chain,
					aura_block_import,
					proposer,
					network.clone(),
					inherent_data_providers.clone(),
					force_authoring,
					backoff_authoring_blocks,
					keystore_container.sync_keystore(),
					can_author_with,
				)?;

				// the AURA authoring task is considered essential, i.e. if it
				// fails we take down the service with it.
				task_manager
					.spawn_essential_handle()
					.spawn_blocking("aura", aura);

				// if the node isn't actively participating in consensus then it doesn't
				// need a keystore, regardless of which protocol we use below.
				let keystore = if role.is_authority() {
					Some(keystore_container.sync_keystore())
				} else {
					None
				};
				let grandpa_config = sc_finality_grandpa::Config {
					// FIXME #1578 make this available through chainspec
					gossip_duration: Duration::from_millis(333),
					justification_period: 512,
					name: Some(name),
					observer_enabled: false,
					keystore,
					is_authority: role.is_network_authority(),
				};

				if enable_grandpa {
					// start the full GRANDPA voter
					// NOTE: non-authorities could run the GRANDPA observer protocol, but at
					// this point the full voter should provide better guarantees of block
					// and vote data availability than the observer. The observer has not
					// been tested extensively yet and having most nodes in a network run it
					// could lead to finality stalls.
					let grandpa_config = sc_finality_grandpa::GrandpaParams {
						config: grandpa_config,
						link: grandpa_link,
						network,
						telemetry_on_connect: Some(telemetry_connection_sinks.on_connect_stream()),
						voting_rule: sc_finality_grandpa::VotingRulesBuilder::default().build(),
						prometheus_registry,
						shared_voter_state: SharedVoterState::empty(),
					};

					// the GRANDPA voter task is considered infallible, i.e.
					// if it fails we take down the service with it.
					task_manager.spawn_essential_handle().spawn_blocking(
						"grandpa-voter",
						sc_finality_grandpa::run_grandpa_voter(grandpa_config)?,
					);
				}
			}
		}
	}

	network_starter.start_network();
	Ok(task_manager)
}

/// Builds a new service for a light client.
pub fn new_light(config: Configuration) -> Result<TaskManager, ServiceError> {
	let (client, backend, keystore_container, mut task_manager, on_demand) =
		sc_service::new_light_parts::<Block, RuntimeApi, Executor>(&config)?;

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = Arc::new(sc_transaction_pool::BasicPool::new_light(
		config.transaction_pool.clone(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
		on_demand.clone(),
	));

	let (grandpa_block_import, _) = sc_finality_grandpa::block_import(
		client.clone(),
		&(client.clone() as Arc<_>),
		select_chain.clone(),
	)?;

	let import_queue = sc_consensus_aura::import_queue::<_, _, _, AuraPair, _, _>(
		sc_consensus_aura::slot_duration(&*client)?,
		grandpa_block_import.clone(),
		Some(Box::new(grandpa_block_import)),
		client.clone(),
		InherentDataProviders::new(),
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
		sp_consensus::NeverCanAuthor,
	)?;

	let light_deps = shadows_rpc::LightDeps {
		remote_blockchain: backend.remote_blockchain(),
		fetcher: on_demand.clone(),
		client: client.clone(),
		pool: transaction_pool.clone(),
	};

	let rpc_extensions = shadows_rpc::create_light(light_deps);

	let (network, network_status_sinks, system_rpc_tx, network_starter) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: Some(on_demand.clone()),
			block_announce_validator_builder: None,
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			backend.clone(),
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		remote_blockchain: Some(backend.remote_blockchain()),
		transaction_pool,
		task_manager: &mut task_manager,
		on_demand: Some(on_demand),
		rpc_extensions_builder: Box::new(sc_service::NoopRpcExtensionBuilder(rpc_extensions)),
		telemetry_connection_sinks: sc_service::TelemetryConnectionSinks::default(),
		config,
		client,
		keystore: keystore_container.sync_keystore(),
		backend,
		network,
		network_status_sinks,
		system_rpc_tx,
	})?;

	network_starter.start_network();

	Ok(task_manager)
}
