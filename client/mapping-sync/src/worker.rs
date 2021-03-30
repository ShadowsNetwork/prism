use std::time::Duration;
use std::pin::Pin;
use std::sync::Arc;
use futures::{prelude::*, task::{Context, Poll}};
use sp_runtime::traits::Block as BlockT;
use sc_client_api::ImportNotifications;
use sp_api::ProvideRuntimeApi;
use sc_client_api::BlockOf;
use sp_blockchain::HeaderBackend;
use fp_rpc::EthereumRuntimeRPCApi;
use futures_timer::Delay;
use log::warn;

const LIMIT: usize = 8;

pub struct MappingSyncWorker<Block: BlockT, C, B> {
	import_notifications: ImportNotifications<Block>,
	timeout: Duration,
	inner_delay: Option<Delay>,

	client: Arc<C>,
	substrate_backend: Arc<B>,
	shadows_backend: Arc<fc_db::Backend<Block>>,

	have_next: bool,
}

impl<Block: BlockT, C, B> MappingSyncWorker<Block, C, B> {
	pub fn new(
		import_notifications: ImportNotifications<Block>,
		timeout: Duration,
		client: Arc<C>,
		substrate_backend: Arc<B>,
		shadows_backend: Arc<fc_db::Backend<Block>>,
	) -> Self {
		Self {
			import_notifications,
			timeout,
			inner_delay: None,

			client,
			substrate_backend,
			shadows_backend,

			have_next: true,
		}
	}
}

impl<Block: BlockT, C, B> Stream for MappingSyncWorker<Block, C, B> where
	C: ProvideRuntimeApi<Block> + Send + Sync + HeaderBackend<Block> + BlockOf,
	C::Api: EthereumRuntimeRPCApi<Block>,
	B: sc_client_api::Backend<Block>,
{
	type Item = ();

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<()>> {
		let mut fire = false;

		loop {
			match Stream::poll_next(Pin::new(&mut self.import_notifications), cx) {
				Poll::Pending => break,
				Poll::Ready(Some(_)) => {
					fire = true;
				},
				Poll::Ready(None) => return Poll::Ready(None),
			}
		}

		let timeout = self.timeout.clone();
		let inner_delay = self.inner_delay.get_or_insert_with(|| Delay::new(timeout));

		match Future::poll(Pin::new(inner_delay), cx) {
			Poll::Pending => (),
			Poll::Ready(()) => {
				fire = true;
			},
		}

		if self.have_next {
			fire = true;
		}

		if fire {
			self.inner_delay = None;

			match crate::sync_blocks(
				self.client.as_ref(),
				self.substrate_backend.blockchain(),
				self.shadows_backend.as_ref(),
				LIMIT,
			) {
				Ok(have_next) => {
					self.have_next = have_next;
					Poll::Ready(Some(()))
				},
				Err(e) => {
					self.have_next = false;
					warn!(target: "mapping-sync", "Syncing failed with error {:?}, retrying.", e);
					Poll::Ready(Some(()))
				},
			}
		} else {
			Poll::Pending
		}
	}
}
