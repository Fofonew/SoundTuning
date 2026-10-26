use std::fmt;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;

use serde::Deserialize;

pub const RELEASES_PAGE_URL: &str = "https://github.com/Fofonew/SoundTuning/releases";
const LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/Fofonew/SoundTuning/releases/latest";
const TRUSTED_PAGE_PREFIX: &str = "https://github.com/Fofonew/SoundTuning/releases/";
const USER_AGENT: &str = concat!("SoundTuning/", env!("CARGO_PKG_VERSION"));
const GITHUB_JSON: &str = "application/vnd.github+json";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(4);
const GLOBAL_TIMEOUT: Duration = Duration::from_secs(10);
const HTTP_NOT_FOUND: u16 = 404;
const MAX_RESPONSE_BYTES: u64 = 256 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        let core = text.trim().trim_start_matches(['v', 'V']);
        let core = core.split_once('+').map_or(core, |(core, _)| core);
        if core.contains('-') {
            return None;
        }
        let mut parts = core.split('.').map(|part| part.parse::<u64>().ok());
        let version = Self::new(parts.next()??, parts.next()??, parts.next()??);
        parts.next().is_none().then_some(version)
    }

    pub fn current() -> Self {
        Self::parse(env!("CARGO_PKG_VERSION")).unwrap_or_default()
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub version: Version,
    pub page_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateStatus {
    UpToDate,
    Available(Release),
    Unreachable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckTrigger {
    Startup,
    Manual,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateSource {
    GitHub,
    Offline,
    Fixed(Option<Release>),
}

#[derive(Deserialize)]
struct LatestReleaseJson {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

pub fn parse_latest_release(body: &str) -> Option<Release> {
    let json: LatestReleaseJson = serde_json::from_str(body).ok()?;
    if json.draft || json.prerelease {
        return None;
    }
    let page_url = if json.html_url.starts_with(TRUSTED_PAGE_PREFIX) {
        json.html_url
    } else {
        RELEASES_PAGE_URL.to_owned()
    };
    Some(Release {
        version: Version::parse(&json.tag_name)?,
        page_url,
    })
}

pub fn status_for(current: Version, latest: Option<Release>) -> UpdateStatus {
    match latest {
        Some(release) if release.version > current => UpdateStatus::Available(release),
        _ => UpdateStatus::UpToDate,
    }
}

pub struct UpdateChecker {
    source: UpdateSource,
    current: Version,
    pending: Option<(CheckTrigger, Receiver<UpdateStatus>)>,
}

impl UpdateChecker {
    pub fn new(source: UpdateSource, current: Version) -> Self {
        Self {
            source,
            current,
            pending: None,
        }
    }

    pub fn is_checking(&self) -> bool {
        self.pending.is_some()
    }

    pub fn start(&mut self, trigger: CheckTrigger) {
        if self.pending.is_some() {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        let current = self.current;
        match &self.source {
            UpdateSource::Offline => {
                let _ = sender.send(UpdateStatus::Unreachable);
            }
            UpdateSource::Fixed(release) => {
                let _ = sender.send(status_for(current, release.clone()));
            }
            UpdateSource::GitHub => {
                let spawned = std::thread::Builder::new()
                    .name("update-check".into())
                    .spawn(move || {
                        let _ = sender.send(check_github(current));
                    });
                if let Err(error) = spawned {
                    tracing::warn!(%error, "could not start the update check");
                }
            }
        }
        self.pending = Some((trigger, receiver));
    }

    pub fn poll(&mut self) -> Option<(CheckTrigger, UpdateStatus)> {
        let (trigger, receiver) = self.pending.as_ref()?;
        let status = match receiver.try_recv() {
            Ok(status) => status,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => UpdateStatus::Unreachable,
        };
        let trigger = *trigger;
        self.pending = None;
        Some((trigger, status))
    }
}

fn check_github(current: Version) -> UpdateStatus {
    match fetch_latest_release() {
        Ok(body) => status_for(current, parse_latest_release(&body)),
        Err(ureq::Error::StatusCode(HTTP_NOT_FOUND)) => UpdateStatus::UpToDate,
        Err(error) => {
            tracing::info!(%error, "update check skipped");
            UpdateStatus::Unreachable
        }
    }
}

fn fetch_latest_release() -> Result<String, ureq::Error> {
    let tls = ureq::tls::TlsConfig::builder()
        .provider(ureq::tls::TlsProvider::NativeTls)
        .root_certs(ureq::tls::RootCerts::PlatformVerifier)
        .build();
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .tls_config(tls)
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_global(Some(GLOBAL_TIMEOUT))
        .user_agent(USER_AGENT)
        .build()
        .into();
    agent
        .get(LATEST_RELEASE_API_URL)
        .header("Accept", GITHUB_JSON)
        .call()?
        .body_mut()
        .with_config()
        .limit(MAX_RESPONSE_BYTES)
        .read_to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release_json(tag: &str, url: &str, prerelease: bool) -> String {
        format!(
            r#"{{"tag_name":"{tag}","html_url":"{url}","draft":false,"prerelease":{prerelease},"assets":[]}}"#
        )
    }

    #[test]
    fn versions_parse_with_or_without_a_v_prefix() {
        assert_eq!(Version::parse("v1.2.3"), Some(Version::new(1, 2, 3)));
        assert_eq!(Version::parse("0.10.0"), Some(Version::new(0, 10, 0)));
        assert_eq!(Version::parse("1.2.3+build"), Some(Version::new(1, 2, 3)));
        for invalid in ["", "v1.2", "1.2.3.4", "1.2.x", "1.2.3-beta.1"] {
            assert_eq!(Version::parse(invalid), None, "{invalid}");
        }
        assert!(Version::new(0, 10, 0) > Version::new(0, 9, 12));
        assert_eq!(Version::current().to_string(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn only_a_newer_published_release_is_offered() {
        let url = "https://github.com/Fofonew/SoundTuning/releases/tag/v0.2.0";
        let release = parse_latest_release(&release_json("v0.2.0", url, false));
        assert_eq!(
            release,
            Some(Release {
                version: Version::new(0, 2, 0),
                page_url: url.to_owned(),
            })
        );
        assert!(matches!(
            status_for(Version::new(0, 1, 0), release.clone()),
            UpdateStatus::Available(_)
        ));
        assert_eq!(
            status_for(Version::new(0, 2, 0), release),
            UpdateStatus::UpToDate
        );
        assert_eq!(
            parse_latest_release(&release_json("v0.3.0", url, true)),
            None
        );
        assert_eq!(parse_latest_release("<html>proxy login</html>"), None);
    }

    #[test]
    fn untrusted_release_links_fall_back_to_the_releases_page() {
        let release =
            parse_latest_release(&release_json("v9.0.0", "file:///etc/passwd", false)).unwrap();
        assert_eq!(release.page_url, RELEASES_PAGE_URL);
    }

    #[test]
    fn offline_and_fixed_sources_answer_without_network() {
        let mut offline = UpdateChecker::new(UpdateSource::Offline, Version::new(0, 1, 0));
        assert_eq!(offline.poll(), None);
        offline.start(CheckTrigger::Startup);
        assert_eq!(
            offline.poll(),
            Some((CheckTrigger::Startup, UpdateStatus::Unreachable))
        );
        assert!(!offline.is_checking());

        let release = Release {
            version: Version::new(0, 2, 0),
            page_url: RELEASES_PAGE_URL.to_owned(),
        };
        let mut fixed = UpdateChecker::new(
            UpdateSource::Fixed(Some(release.clone())),
            Version::new(0, 1, 0),
        );
        fixed.start(CheckTrigger::Manual);
        assert_eq!(
            fixed.poll(),
            Some((CheckTrigger::Manual, UpdateStatus::Available(release)))
        );
    }
}
