use crate::core::Account;

/// Snapshot of current network and schedule state, provided by the platform/service layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvironmentStatus {
    /// Whether the current network connection is metered (e.g. cellular, tethered).
    pub is_metered: bool,
    /// Whether a VPN tunnel is currently active.
    pub vpn_active: bool,
    /// Whether the current time falls within the global sync schedule's off-hours.
    pub is_off_hours: bool,
}

/// Reasons why sync is currently suppressed for an account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncPauseReason {
    /// Sync is globally disabled for this account.
    SyncDisabled,
    /// Account requires an unmetered connection but the network is metered.
    MeteredConnection,
    /// Account requires a VPN but none is active.
    NoVpn,
    /// Current time is off-hours and this account is not exempt from the schedule.
    OffHoursSchedule,
}

impl SyncPauseReason {
    /// Human-readable description of this pause reason.
    pub fn description(&self) -> &'static str {
        match self {
            Self::SyncDisabled => "Synchronization is disabled",
            Self::MeteredConnection => "Paused: metered connection",
            Self::NoVpn => "Paused: no VPN active",
            Self::OffHoursSchedule => "Paused: outside sync schedule",
        }
    }
}

/// Result of evaluating whether an account is eligible to sync right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncEligibility {
    /// Empty means sync may proceed; non-empty means sync is suppressed.
    pub pause_reasons: Vec<SyncPauseReason>,
}

impl SyncEligibility {
    /// Whether all conditions are satisfied and sync may proceed.
    pub fn can_sync(&self) -> bool {
        self.pause_reasons.is_empty()
    }
}

/// Evaluate whether the given account is eligible to sync under the current environment.
///
/// All conditions are evaluated independently — every unsatisfied condition is reported.
pub fn evaluate(account: &Account, env: &EnvironmentStatus) -> SyncEligibility {
    let mut reasons = Vec::new();

    if !account.sync_enabled() {
        reasons.push(SyncPauseReason::SyncDisabled);
    }

    if account.unmetered_only() && env.is_metered {
        reasons.push(SyncPauseReason::MeteredConnection);
    }

    if account.vpn_only() && !env.vpn_active {
        reasons.push(SyncPauseReason::NoVpn);
    }

    if env.is_off_hours && !account.schedule_exempt() {
        reasons.push(SyncPauseReason::OffHoursSchedule);
    }

    SyncEligibility {
        pause_reasons: reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{AuthMethod, EncryptionMode, NewAccountParams, Protocol};

    fn make_account(unmetered_only: bool, vpn_only: bool, schedule_exempt: bool) -> Account {
        Account::new(NewAccountParams {
            display_name: "Test".into(),
            protocol: Protocol::Imap,
            host: "imap.example.com".into(),
            port: 993,
            encryption: EncryptionMode::SslTls,
            auth_method: AuthMethod::Plain,
            username: "user@example.com".into(),
            credential: "secret".into(),
            smtp: None,
            pop3_settings: None,
            color: None,
            avatar_path: None,
            category: None,
            sync_enabled: true,
            on_demand: false,
            polling_interval_minutes: None,
            unmetered_only,
            vpn_only,
            schedule_exempt,
            system_folders: None,
            swipe_defaults: None,
            notifications_enabled: true,
            security_settings: None,
        })
        .unwrap()
    }

    fn normal_env() -> EnvironmentStatus {
        EnvironmentStatus {
            is_metered: false,
            vpn_active: true,
            is_off_hours: false,
        }
    }

    // -- Basic: all conditions met --

    #[test]
    fn all_conditions_met_allows_sync() {
        let acct = make_account(true, true, false);
        let env = normal_env();
        let result = evaluate(&acct, &env);
        assert!(result.can_sync());
        assert!(result.pause_reasons.is_empty());
    }

    #[test]
    fn default_account_always_syncs() {
        let acct = make_account(false, false, false);
        let env = normal_env();
        assert!(evaluate(&acct, &env).can_sync());
    }

    // -- Sync disabled --

    #[test]
    fn sync_disabled_blocks_sync() {
        let mut acct = make_account(false, false, false);
        acct.set_sync_enabled(false);
        let result = evaluate(&acct, &normal_env());
        assert!(!result.can_sync());
        assert!(result
            .pause_reasons
            .contains(&SyncPauseReason::SyncDisabled));
    }

    // -- Metered connection --

    #[test]
    fn unmetered_only_blocks_on_metered() {
        let acct = make_account(true, false, false);
        let env = EnvironmentStatus {
            is_metered: true,
            vpn_active: false,
            is_off_hours: false,
        };
        let result = evaluate(&acct, &env);
        assert!(!result.can_sync());
        assert!(result
            .pause_reasons
            .contains(&SyncPauseReason::MeteredConnection));
    }

    #[test]
    fn unmetered_only_allows_on_unmetered() {
        let acct = make_account(true, false, false);
        let env = EnvironmentStatus {
            is_metered: false,
            vpn_active: false,
            is_off_hours: false,
        };
        assert!(evaluate(&acct, &env).can_sync());
    }

    #[test]
    fn non_unmetered_account_ignores_metered_network() {
        let acct = make_account(false, false, false);
        let env = EnvironmentStatus {
            is_metered: true,
            vpn_active: false,
            is_off_hours: false,
        };
        assert!(evaluate(&acct, &env).can_sync());
    }

    // -- VPN --

    #[test]
    fn vpn_only_blocks_without_vpn() {
        let acct = make_account(false, true, false);
        let env = EnvironmentStatus {
            is_metered: false,
            vpn_active: false,
            is_off_hours: false,
        };
        let result = evaluate(&acct, &env);
        assert!(!result.can_sync());
        assert!(result.pause_reasons.contains(&SyncPauseReason::NoVpn));
    }

    #[test]
    fn vpn_only_allows_with_vpn() {
        let acct = make_account(false, true, false);
        let env = EnvironmentStatus {
            is_metered: false,
            vpn_active: true,
            is_off_hours: false,
        };
        assert!(evaluate(&acct, &env).can_sync());
    }

    #[test]
    fn non_vpn_account_ignores_vpn_state() {
        let acct = make_account(false, false, false);
        let env = EnvironmentStatus {
            is_metered: false,
            vpn_active: false,
            is_off_hours: false,
        };
        assert!(evaluate(&acct, &env).can_sync());
    }

    // -- Schedule exemption --

    #[test]
    fn off_hours_blocks_non_exempt_account() {
        let acct = make_account(false, false, false);
        let env = EnvironmentStatus {
            is_metered: false,
            vpn_active: false,
            is_off_hours: true,
        };
        let result = evaluate(&acct, &env);
        assert!(!result.can_sync());
        assert!(result
            .pause_reasons
            .contains(&SyncPauseReason::OffHoursSchedule));
    }

    #[test]
    fn off_hours_allows_exempt_account() {
        let acct = make_account(false, false, true);
        let env = EnvironmentStatus {
            is_metered: false,
            vpn_active: false,
            is_off_hours: true,
        };
        assert!(evaluate(&acct, &env).can_sync());
    }

    #[test]
    fn non_off_hours_ignores_exempt_flag() {
        let acct = make_account(false, false, false);
        let env = EnvironmentStatus {
            is_metered: false,
            vpn_active: false,
            is_off_hours: false,
        };
        assert!(evaluate(&acct, &env).can_sync());
    }

    // -- Multiple conditions --

    #[test]
    fn multiple_conditions_all_reported() {
        let mut acct = make_account(true, true, false);
        acct.set_sync_enabled(false);
        let env = EnvironmentStatus {
            is_metered: true,
            vpn_active: false,
            is_off_hours: true,
        };
        let result = evaluate(&acct, &env);
        assert!(!result.can_sync());
        assert_eq!(result.pause_reasons.len(), 4);
        assert!(result
            .pause_reasons
            .contains(&SyncPauseReason::SyncDisabled));
        assert!(result
            .pause_reasons
            .contains(&SyncPauseReason::MeteredConnection));
        assert!(result.pause_reasons.contains(&SyncPauseReason::NoVpn));
        assert!(result
            .pause_reasons
            .contains(&SyncPauseReason::OffHoursSchedule));
    }

    #[test]
    fn metered_and_no_vpn_both_reported() {
        let acct = make_account(true, true, false);
        let env = EnvironmentStatus {
            is_metered: true,
            vpn_active: false,
            is_off_hours: false,
        };
        let result = evaluate(&acct, &env);
        assert!(!result.can_sync());
        assert_eq!(result.pause_reasons.len(), 2);
        assert!(result
            .pause_reasons
            .contains(&SyncPauseReason::MeteredConnection));
        assert!(result.pause_reasons.contains(&SyncPauseReason::NoVpn));
    }

    // -- Pause reason descriptions --

    #[test]
    fn pause_reasons_have_descriptions() {
        assert!(!SyncPauseReason::SyncDisabled.description().is_empty());
        assert!(!SyncPauseReason::MeteredConnection.description().is_empty());
        assert!(!SyncPauseReason::NoVpn.description().is_empty());
        assert!(!SyncPauseReason::OffHoursSchedule.description().is_empty());
    }

    // -- Conditions are evaluated independently --

    #[test]
    fn schedule_exempt_does_not_affect_network_conditions() {
        let acct = make_account(true, true, true);
        let env = EnvironmentStatus {
            is_metered: true,
            vpn_active: false,
            is_off_hours: true,
        };
        let result = evaluate(&acct, &env);
        // Schedule exempt removes off-hours but metered + no VPN still block.
        assert!(!result.can_sync());
        assert_eq!(result.pause_reasons.len(), 2);
        assert!(!result
            .pause_reasons
            .contains(&SyncPauseReason::OffHoursSchedule));
    }
}
