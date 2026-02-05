use crate::db::Database;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
#[schema(as = PlatformProject)]
pub struct PlatformProject {
    /// Unique project identifier
    #[schema(example = 1)]
    pub id: i64,
    /// Human-readable project name
    #[schema(example = "Acme E-commerce Platform")]
    pub name: String,
    /// URL-friendly project identifier (unique)
    #[schema(example = "acme-ecommerce")]
    pub slug: String,
    /// Project status: active or suspended
    #[schema(example = "active", enum_values = ["active", "suspended"])]
    pub status: String,
    /// Subscription plan: dev, pro, or enterprise
    #[schema(example = "pro", enum_values = ["dev", "pro", "enterprise"])]
    pub plan: String,
    /// Deployment region
    #[schema(example = "us-east-1")]
    pub region: String,
    /// PostgreSQL database connection URL
    #[schema(example = "postgresql://postgres:password@db.example.com:5432/mydb")]
    pub db_url: String,
    /// Supabase API base URL
    #[schema(example = "https://api.example.com")]
    pub api_base_url: String,
    /// Project creation timestamp
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[schema(as = CreatePlatformProject)]
pub struct CreatePlatformProject {
    /// Human-readable project name
    #[schema(example = "Acme E-commerce Platform")]
    pub name: String,
    /// URL-friendly project identifier (must be unique)
    #[schema(example = "acme-ecommerce")]
    pub slug: String,
    /// Subscription plan: dev, pro, or enterprise
    #[schema(example = "pro", enum_values = ["dev", "pro", "enterprise"])]
    pub plan: String,
    /// Deployment region
    #[schema(example = "us-east-1")]
    pub region: String,
    /// PostgreSQL database connection URL
    #[schema(example = "postgresql://postgres:password@db.example.com:5432/mydb")]
    pub db_url: String,
    /// Supabase API base URL
    #[schema(example = "https://api.example.com")]
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


