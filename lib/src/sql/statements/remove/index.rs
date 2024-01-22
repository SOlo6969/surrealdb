use crate::ctx::Context;
use crate::dbs::{Options, Transaction};
use crate::err::Error;
use crate::iam::{Action, ResourceKind};
use crate::sql::{Base, Ident, Value};
use derive::Store;
use revision::revisioned;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Store, Hash)]
#[revisioned(revision = 2)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct RemoveIndexStatement {
	pub name: Ident,
	pub what: Ident,
	#[revision(start = 2)]
	pub if_exists: bool,
}

impl RemoveIndexStatement {
	/// Process this type returning a computed simple Value
	pub(crate) async fn compute(
		&self,
		ctx: &Context<'_>,
		opt: &Options,
		txn: &Transaction,
	) -> Result<Value, Error> {
		// Allowed to run?
		opt.is_allowed(Action::Edit, ResourceKind::Index, &Base::Db)?;
		// Claim transaction
		let mut run = txn.lock().await;
		// Clear the index store cache
		ctx.get_index_stores().index_removed(opt, &mut run, &self.what, &self.name).await?;
		// Clear the cache
		run.clear_cache();
		match run.get_tb_index(opt.ns(), opt.db(), &self.what, &self.name).await {
			Ok(ix) => {
				let ix_name = ix.name.to_string();
				// Delete the definition
				let key = crate::key::table::ix::new(opt.ns(), opt.db(), &ix.what, &ix_name);
				run.del(key).await?;
				// Remove the index data
				let key = crate::key::index::all::new(opt.ns(), opt.db(), &ix.what, &ix_name);
				run.delp(key, u32::MAX).await?;
				// Clear the cache
				let key = crate::key::table::ix::prefix(opt.ns(), opt.db(), &ix.what);
				run.clr(key).await?;
				// Ok all good
				Ok(Value::None)
			}
			Err(err) => {
				if matches!(err, Error::IxNotFound { .. }) && self.if_exists {
					Ok(Value::None)
				} else {
					Err(err)
				}
			}
		}
	}
}

impl Display for RemoveIndexStatement {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "REMOVE INDEX {} ON {}", self.name, self.what)?;
		if self.if_exists {
			write!(f, " IF EXISTS")?
		}
		Ok(())
	}
}
