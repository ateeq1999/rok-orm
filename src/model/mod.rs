#[allow(clippy::module_inception)]
mod model;
mod prunable;
#[cfg(feature = "postgres")]
mod pg_model;
#[cfg(feature = "postgres")]
mod pg_model_ext;
#[cfg(feature = "sqlite")]
mod sqlite_model;
#[cfg(feature = "sqlite")]
mod sqlite_model_ext;
#[cfg(feature = "mysql")]
mod mysql_model;
#[cfg(feature = "mysql")]
mod mysql_model_ext;

pub use model::{Model, timestamps_muted, events_muted};
pub use prunable::{Prunable, PrunableRegistry};
#[cfg(feature = "postgres")]
pub use pg_model::PgModel;
#[cfg(feature = "postgres")]
pub use pg_model_ext::PgModelExt;
#[cfg(feature = "sqlite")]
pub use sqlite_model::SqliteModel;
#[cfg(feature = "sqlite")]
pub use sqlite_model_ext::SqliteModelExt;
#[cfg(feature = "mysql")]
pub use mysql_model::MyModel;
#[cfg(feature = "mysql")]
pub use mysql_model_ext::MyModelExt;
