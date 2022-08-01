use jsonrpsee::{
	core::{async_trait, Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use pallet_dex_runtime_api::DexRuntimeApi;
use sp_api::{BlockId, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

#[rpc(client, server)]
pub trait DexApi {
	/// Get the current price of a market
	///
	/// # Arguments:
	/// market: (BASE AssetId, QUOTE AssetId), TODO: Strings could be nice here
	///
	/// # Returns:
	/// If Ok, the current price for the market
	/// Else some error
	#[method(name = "dex_currentPrice")]
	async fn current_price(&self, market: (u8, u8)) -> RpcResult<f64>;
}

pub struct Dex<C, Block> {
	client: Arc<C>,
	_market: std::marker::PhantomData<Block>,
}

impl<C, Block> Dex<C, Block> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _market: Default::default() }
	}
}

#[async_trait]
impl<C, Block> DexApiServer for Dex<C, Block>
where
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: DexRuntimeApi<Block>,
{
	async fn current_price(&self, market: (u8, u8)) -> RpcResult<f64> {
		let api = self.client.runtime_api();

		// Just take the latest best block
		let at = BlockId::hash(self.client.info().best_hash);
		let (numerator, denominator) =
			api.current_price(&at, market).map_err(|_e| Error::RuntimeCall)?;

		Ok((numerator as f64 / denominator as f64))
	}
}

/// Just a quick error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Runtime call returned an error")]
	RuntimeCall,
}

impl From<Error> for JsonRpseeError {
	fn from(error: Error) -> Self {
		let message = error.to_string();
		JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(1234, message, None::<()>)))
	}
}
