use crate::db::Database;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlatformProject {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub status: String,
    pub plan: String,
    pub region: String,
    pub db_url: String,
    pub api_base_url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePlatformProject {
    pub name: String,
    pub slug: String,
    pub plan: String,
    pub region: String,
    pub db_url: String,
    pub api_base_url: String,
}

impl Database {
    pub async fn create_platform_project(
        &self,
        input: CreatePlatformProject,
    ) -> anyhow::Result<PlatformProject> {
        let project = sqlx::query_as::<_, PlatformProject>(
            r#"
            INSERT INTO platform_projects (name, slug, status, plan, region, db_url, api_base_url)
            VALUES ($1, $2, 'active', $3, $4, $5, $6)
            RETURNING id, name, slug, status, plan, region, db_url, api_base_url, created_at
            "#,
        )
        .bind(&input.name)
        .bind(&input.slug)
        .bind(&input.plan)
        .bind(&input.region)
        .bind(&input.db_url)
        .bind(&input.api_base_url)
        .fetch_one(&self.pool)
        .await?;

        Ok(project)
    }

    pub async fn list_platform_projects(&self) -> anyhow::Result<Vec<PlatformProject>> {
        let projects = sqlx::query_as::<_, PlatformProject>(
            r#"
            SELECT id, name, slug, status, plan, region, db_url, api_base_url, created_at
            FROM platform_projects
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(projects)
    }

    pub async fn update_platform_project_status(
        &self,
        id: i64,
        status: &str,
    ) -> anyhow::Result<Option<PlatformProject>> {
        let project = sqlx::query_as::<_, PlatformProject>(
            r#"
            UPDATE platform_projects
            SET status = $2
            WHERE id = $1
            RETURNING id, name, slug, status, plan, region, db_url, api_base_url, created_at
            "#,
        )
        .bind(id)
        .bind(status)
        .fetch_optional(&self.pool)
        .await?;

        Ok(project)
    }
}


