use async_graphql::dataloader::{DataLoader, LruCache};
use db::loaders::{
    office::OfficeLoader, organization::OrganizationLoader, politician::PoliticianLoader,
    race::RaceLoader,
};
use sqlx::PgPool;

pub struct ApiContext {
    pub pool: PgPool,
    pub loaders: DataLoaders,
}

pub struct DataLoaders {
    pub organization_loader: DataLoader<OrganizationLoader, LruCache>,
    pub politician_loader: DataLoader<PoliticianLoader, LruCache>,
    pub office_loader: DataLoader<OfficeLoader, LruCache>,
    pub race_loader: DataLoader<RaceLoader, LruCache>,
}

impl DataLoaders {
    pub fn new(pool: PgPool) -> Self {
        Self {
            organization_loader: DataLoader::with_cache(
                OrganizationLoader::new(pool.clone()),
                tokio::task::spawn,
                LruCache::new(64),
            ),
            politician_loader: DataLoader::with_cache(
                PoliticianLoader::new(pool.clone()),
                tokio::task::spawn,
                LruCache::new(64),
            ),
            office_loader: DataLoader::with_cache(
                OfficeLoader::new(pool.clone()),
                tokio::task::spawn,
                LruCache::new(64),
            ),
            race_loader: DataLoader::with_cache(
                RaceLoader::new(pool),
                tokio::task::spawn,
                LruCache::new(64),
            ),
        }
    }
}

impl ApiContext {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: pool.clone(),
            loaders: DataLoaders::new(pool),
        }
    }
}
