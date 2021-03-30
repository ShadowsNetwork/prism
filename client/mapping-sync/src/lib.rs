mod worker;

pub use worker::MappingSyncWorker;

use sp_runtime::{generic::BlockId, traits::{Block as BlockT, Header as HeaderT, Zero}};
use sp_api::ProvideRuntimeApi;
use sc_client_api::BlockOf;
use sp_blockchain::HeaderBackend;
use fp_rpc::EthereumRuntimeRPCApi;

pub fn sync_block<Block: BlockT>(
	backend: &fc_db::Backend<Block>,
	header: &Block::Header,
) -> Result<(), String> {
	let log = fp_consensus::find_log(header.digest()).map_err(|e| format!("{:?}", e))?;
	let post_hashes = log.into_hashes();

	let mapping_commitment = fc_db::MappingCommitment {
		block_hash: header.hash(),
		ethereum_block_hash: post_hashes.block_hash,
		ethereum_transaction_hashes: post_hashes.transaction_hashes,
	};
	backend.mapping().write_hashes(mapping_commitment)?;

	Ok(())
}

pub fn sync_genesis_block<Block: BlockT, C>(
	client: &C,
	backend: &fc_db::Backend<Block>,
	header: &Block::Header,
) -> Result<(), String> where
	C: ProvideRuntimeApi<Block> + Send + Sync + HeaderBackend<Block> + BlockOf,
	C::Api: EthereumRuntimeRPCApi<Block>,
{
	let id = BlockId::Hash(header.hash());

	let block = client.runtime_api().current_block(&id)
		.map_err(|e| format!("{:?}", e))?;
	let block_hash = block.ok_or("Ethereum genesis block not found".to_string())?.header.hash();
	let mapping_commitment = fc_db::MappingCommitment::<Block> {
		block_hash: header.hash(),
		ethereum_block_hash: block_hash,
		ethereum_transaction_hashes: Vec::new(),
	};
	backend.mapping().write_hashes(mapping_commitment)?;

	Ok(())
}

pub fn sync_one_block<Block: BlockT, C, B>(
	client: &C,
	substrate_backend: &B,
	shadows_backend: &fc_db::Backend<Block>,
) -> Result<bool, String> where
	C: ProvideRuntimeApi<Block> + Send + Sync + HeaderBackend<Block> + BlockOf,
	C::Api: EthereumRuntimeRPCApi<Block>,
	B: sp_blockchain::HeaderBackend<Block> + sp_blockchain::Backend<Block>,
{
	let mut current_syncing_tips = shadows_backend.meta().current_syncing_tips()?;

	if current_syncing_tips.is_empty() {
		let mut leaves = substrate_backend.leaves().map_err(|e| format!("{:?}", e))?;
		if leaves.is_empty() {
			return Ok(false)
		}

		current_syncing_tips.append(&mut leaves);
	}

	let mut operating_tip = None;

	while let Some(checking_tip) = current_syncing_tips.pop() {
		if !shadows_backend.mapping().is_synced(&checking_tip).map_err(|e| format!("{:?}", e))? {
			operating_tip = Some(checking_tip);
			break
		}
	}

	let operating_tip = match operating_tip {
		Some(operating_tip) => operating_tip,
		None => {
			shadows_backend.meta().write_current_syncing_tips(current_syncing_tips)?;
			return Ok(false)
		}
	};

	let operating_header = substrate_backend.header(BlockId::Hash(operating_tip))
		.map_err(|e| format!("{:?}", e))?
		.ok_or("Header not found".to_string())?;

	if operating_header.number() == &Zero::zero() {
		sync_genesis_block(client, shadows_backend, &operating_header)?;

		shadows_backend.meta().write_current_syncing_tips(current_syncing_tips)?;
		Ok(true)
	} else {
		sync_block(shadows_backend, &operating_header)?;

		current_syncing_tips.push(*operating_header.parent_hash());
		shadows_backend.meta().write_current_syncing_tips(current_syncing_tips)?;
		Ok(true)
	}
}

pub fn sync_blocks<Block: BlockT, C, B>(
	client: &C,
	substrate_backend: &B,
	shadows_backend: &fc_db::Backend<Block>,
	limit: usize,
) -> Result<bool, String> where
	C: ProvideRuntimeApi<Block> + Send + Sync + HeaderBackend<Block> + BlockOf,
	C::Api: EthereumRuntimeRPCApi<Block>,
	B: sp_blockchain::HeaderBackend<Block> + sp_blockchain::Backend<Block>,
{
	let mut synced_any = false;

	for _ in 0..limit {
		synced_any = synced_any || sync_one_block(client, substrate_backend, shadows_backend)?;
	}

	Ok(synced_any)
}
