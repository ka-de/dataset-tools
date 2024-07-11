// g/src/lib.rs

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

//! A library for interacting with GitHub and Redis.
//!
//! This library provides functionality to interact with GitHub repositories,
//! create gists, and store repository information in Redis. It uses the
//! `octocrab` crate for GitHub API interactions and the `redis` crate for
//! Redis operations.

use octocrab::Octocrab;
use ::redis::aio;
use ::redis::AsyncCommands;
use std::error::Error;
use url::Url;

pub mod github {
    use super::{ AsyncCommands, Error, Octocrab, Url };

    /// Retrieves statistics for a specific GitHub repository.
    ///
    /// # Arguments
    ///
    /// * `octocrab` - An instance of the Octocrab client.
    /// * `owner` - The owner of the repository.
    /// * `repo` - The name of the repository.
    ///
    /// # Returns
    ///
    /// A tuple containing the full name of the repository, number of stars,
    /// and health percentage.
    ///
    /// # Errors
    ///
    /// This function will return an error if the Octocrab instance fails to fetch the repository statistics.
    pub async fn get_repo_stats(
        octocrab: &Octocrab,
        owner: &str,
        repo: &str
    ) -> Result<(String, u32, u8), Box<dyn Error>> {
        let repo_info = octocrab.repos(owner, repo).get().await?;
        let repo_metrics = octocrab.repos(owner, repo).get_community_profile_metrics().await?;

        let full_name = repo_info.full_name.unwrap_or_else(|| format!("{owner}/{repo}"));
        let stars = repo_info.stargazers_count.unwrap_or_default();
        let Ok(health_percentage) = repo_metrics.health_percentage.try_into() else {
            return Err(
                Box::new(
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to convert health percentage"
                    )
                )
            );
        };

        Ok((full_name, stars, health_percentage))
    }

    /// Stores repository information in Redis.
    ///
    /// # Arguments
    ///
    /// * `con` - A mutable reference to a Redis multiplexed connection.
    /// * `repos` - A slice of tuples containing repository names and URLs.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the Redis operation fails.
    pub async fn store_repos(
        con: &mut ::redis::aio::MultiplexedConnection,
        repos: &[(String, String)]
    ) -> Result<(), Box<dyn Error>> {
        for (name, url) in repos {
            let _: () = con.hset("github_repos", name, url).await?;
        }
        Ok(())
    }

    /// Creates a new GitHub gist.
    ///
    /// # Arguments
    ///
    /// * `octocrab` - An instance of the Octocrab client.
    /// * `file_name` - The name of the file to be included in the gist.
    /// * `content` - The content of the gist.
    /// * `description` - A description of the gist.
    /// * `is_public` - Whether the gist should be public or private.
    ///
    /// # Returns
    ///
    /// The URL of the created gist.
    ///
    /// # Errors
    ///
    /// This function will return an error if the gist creation fails.
    pub async fn create_gist(
        octocrab: &Octocrab,
        file_name: &str,
        content: &str,
        description: &str,
        is_public: bool
    ) -> Result<String, Box<dyn Error>> {
        let gist = octocrab
            .gists()
            .create()
            .file(file_name, content)
            .description(description)
            .public(is_public)
            .send().await?;

        Ok(gist.html_url.to_string())
    }

    /// Lists repositories for the authenticated user.
    ///
    /// # Arguments
    ///
    /// * `octocrab` - An instance of the Octocrab client.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing repository names and URLs.
    ///
    /// # Errors
    ///
    /// This function will return an error if the repository listing fails.
    pub async fn list_repos(octocrab: &Octocrab) -> Result<Vec<(String, String)>, Box<dyn Error>> {
        let mut repos = Vec::new();
        let mut page = octocrab
            .current()
            .list_repos_for_authenticated_user()
            .per_page(100)
            .send().await?;

        loop {
            for repo in &page.items {
                let name = repo.name.clone();
                let url = match &repo.html_url {
                    Some(url) => url.to_string(),
                    None =>
                        match Url::parse("https://github.com") {
                            Ok(url) => url.to_string(),
                            Err(_) => {
                                return Err(
                                    Box::new(
                                        std::io::Error::new(
                                            std::io::ErrorKind::Other,
                                            "Failed to parse URL"
                                        )
                                    )
                                );
                            }
                        }
                };
                repos.push((name, url));
            }

            if let Some(next_page) = octocrab.get_page(&page.next).await? {
                page = next_page;
            } else {
                break;
            }
        }

        Ok(repos)
    }
}

pub mod redis {
    use super::{ AsyncCommands, Error, aio, redis };

    /// Stores repository information in Redis.
    ///
    /// # Arguments
    ///
    /// * `con` - A mutable reference to a Redis multiplexed connection.
    /// * `repos` - A slice of tuples containing repository names and URLs.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure of the operation.
    /// # Errors
    ///
    /// This function will return an error if the Redis operation fails.
    pub async fn store_repos(
        con: &mut redis::aio::MultiplexedConnection,
        repos: &[(String, String)]
    ) -> Result<(), Box<dyn Error>> {
        for (name, url) in repos {
            let _: () = con.hset("github_repos", name, url).await?;
        }
        Ok(())
    }
}

/// Builds an Octocrab instance with the provided GitHub token.
///
/// # Arguments
///
/// * `token` - A GitHub personal access token.
///
/// # Returns
///
/// An Octocrab instance configured with the provided token.
///
/// # Errors
///
/// This function will return an error if the Octocrab instance creation fails.
pub fn build_octocrab(token: &str) -> Result<Octocrab, Box<dyn Error>> {
    Ok(Octocrab::builder().personal_token(token.to_string()).build()?)
}
