use async_graphql::dataloader::{DataLoader, LruCache, NoCache};
use db::loaders::{
    issue_tag::IssueTagLoader, office::OfficeLoader, organization::OrganizationLoader,
    politician::PoliticianLoader, race::RaceLoader,
};
use sqlx::PgPool;

pub struct ApiContext {
    pub pool: PgPool,
    pub loaders: DataLoaders,
}

pub struct DataLoaders {
    pub organization_loader: DataLoader<OrganizationLoader, LruCache>,
    pub politician_loader: DataLoader<PoliticianLoader, NoCache>,
    pub office_loader: DataLoader<OfficeLoader, LruCache>,
    pub race_loader: DataLoader<RaceLoader, LruCache>,
    pub issue_tag_loader: DataLoader<IssueTagLoader, LruCache>,
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
                NoCache,
            ),
            office_loader: DataLoader::with_cache(
                OfficeLoader::new(pool.clone()),
                tokio::task::spawn,
                LruCache::new(64),
            ),
            race_loader: DataLoader::with_cache(
                RaceLoader::new(pool.clone()),
                tokio::task::spawn,
                LruCache::new(64),
            ),
            issue_tag_loader: DataLoader::with_cache(
                IssueTagLoader::new(pool),
                tokio::task::spawn,
                LruCache::new(128),
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
