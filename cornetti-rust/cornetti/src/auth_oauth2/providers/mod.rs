pub mod apple;
pub mod custom;
pub mod facebook;
pub mod github;
pub mod google;
pub mod microsoft;

use crate::auth_oauth2::models::OAuth2UserTransportData;
use crate::auth_oauth2::traits::OAuth2Provider;
use crate::core::models::CornettiResult;

/// Enum di tutti i provider built-in.
/// Usato per istanziare il provider giusto dato il nome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinProvider {
    Google,
    GitHub,
    Microsoft,
    Apple,
    Facebook,
}

impl BuiltinProvider {
    /// Risolve un provider dal nome stringa.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "google" => Some(Self::Google),
            "github" => Some(Self::GitHub),
            "microsoft" => Some(Self::Microsoft),
            "apple" => Some(Self::Apple),
            "facebook" => Some(Self::Facebook),
            _ => None,
        }
    }

    /// Restituisce il nome del provider.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Google => google::GoogleOAuth2Provider::name(),
            Self::GitHub => github::GitHubOAuth2Provider::name(),
            Self::Microsoft => microsoft::MicrosoftOAuth2Provider::name(),
            Self::Apple => apple::AppleOAuth2Provider::name(),
            Self::Facebook => facebook::FacebookOAuth2Provider::name(),
        }
    }

    /// Restituisce l'auth URL del provider.
    pub fn auth_url(&self) -> &'static str {
        match self {
            Self::Google => google::GoogleOAuth2Provider::auth_url(),
            Self::GitHub => github::GitHubOAuth2Provider::auth_url(),
            Self::Microsoft => microsoft::MicrosoftOAuth2Provider::auth_url(),
            Self::Apple => apple::AppleOAuth2Provider::auth_url(),
            Self::Facebook => facebook::FacebookOAuth2Provider::auth_url(),
        }
    }

    /// Restituisce il token URL del provider.
    pub fn token_url(&self) -> &'static str {
        match self {
            Self::Google => google::GoogleOAuth2Provider::token_url(),
            Self::GitHub => github::GitHubOAuth2Provider::token_url(),
            Self::Microsoft => microsoft::MicrosoftOAuth2Provider::token_url(),
            Self::Apple => apple::AppleOAuth2Provider::token_url(),
            Self::Facebook => facebook::FacebookOAuth2Provider::token_url(),
        }
    }

    /// Restituisce gli scope di default del provider.
    pub fn default_scopes(&self) -> &'static [&'static str] {
        match self {
            Self::Google => google::GoogleOAuth2Provider::default_scopes(),
            Self::GitHub => github::GitHubOAuth2Provider::default_scopes(),
            Self::Microsoft => microsoft::MicrosoftOAuth2Provider::default_scopes(),
            Self::Apple => apple::AppleOAuth2Provider::default_scopes(),
            Self::Facebook => facebook::FacebookOAuth2Provider::default_scopes(),
        }
    }

    /// Indica se il provider espone un endpoint userinfo.
    pub fn supports_userinfo(&self) -> bool {
        match self {
            Self::Google => google::GoogleOAuth2Provider::supports_userinfo(),
            Self::GitHub => github::GitHubOAuth2Provider::supports_userinfo(),
            Self::Microsoft => microsoft::MicrosoftOAuth2Provider::supports_userinfo(),
            Self::Apple => apple::AppleOAuth2Provider::supports_userinfo(),
            Self::Facebook => facebook::FacebookOAuth2Provider::supports_userinfo(),
        }
    }

    /// Recupera i dati utente dal provider.
    pub async fn get_user_info(
        &self,
        http_client: &reqwest::Client,
        access_token: &str,
    ) -> CornettiResult<OAuth2UserTransportData> {
        match self {
            Self::Google => {
                google::GoogleOAuth2Provider::get_user_info(http_client, access_token).await
            }
            Self::GitHub => {
                github::GitHubOAuth2Provider::get_user_info(http_client, access_token).await
            }
            Self::Microsoft => {
                microsoft::MicrosoftOAuth2Provider::get_user_info(http_client, access_token).await
            }
            Self::Apple => apple::AppleOAuth2Provider::get_user_info(http_client, access_token).await,
            Self::Facebook => {
                facebook::FacebookOAuth2Provider::get_user_info(http_client, access_token).await
            }
        }
    }
}
