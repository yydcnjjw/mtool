mod container;
mod injectable;
mod provider;

pub use container::*;
pub use injectable::*;
pub use provider::*;

use std::any::type_name;

use anyhow::Context;

pub async fn inject<Func, Args, Output, C>(c: &C, f: &Func) -> Result<Output, anyhow::Error>
where
    Func: Inject<Args, Output = Output>,
    Args: Provide<C>,
{
    Ok(f.inject(
        Args::provide(c)
            .await
            .context(format!("Failed to inject {}", type_name::<Args>()))?,
    ))
}

pub async fn inject_once<Func, Args, Output, C>(c: &C, f: Func) -> Result<Output, anyhow::Error>
where
    Func: InjectOnce<Args, Output = Output>,
    Args: Provide<C>,
{
    Ok(f.inject_once(
        Args::provide(c)
            .await
            .context(format!("Failed to inject once {}", type_name::<Args>()))?,
    ))
}
