use async_graphql::dataloader::{DataLoader, HashMapCache};
use db::loaders::{organization::OrganizationLoader, politician::PoliticianLoader};
use sqlx::PgPool;

pub struct ApiContext {
    pub pool: PgPool,
    pub loaders: DataLoaders,
}

pub struct DataLoaders {
    pub organization_loader: DataLoader<OrganizationLoader, HashMapCache>,
    pub politician_loader: DataLoader<PoliticianLoader, HashMapCache>,
}

impl DataLoaders {
    pub fn new(pool: PgPool) -> Self {
        Self {
            organization_loader: DataLoader::with_cache(
                OrganizationLoader::new(pool.clone()),
                tokio::task::spawn,
                HashMapCache::default(),
            ),
            politician_loader: DataLoader::with_cache(
                PoliticianLoader::new(pool.clone()),
                tokio::task::spawn,
                HashMapCache::default(),
            ),
        }
    }
}

impl ApiContext {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: pool.clone(),
            loaders: DataLoaders::new(pool.clone()),
        }
    }
}
