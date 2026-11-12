//! `RankFlags`, `RankDefinition` and `Guild.NewGuildSystem`.
//!
//! `AllianceInfo` and the `Guild` class are in `guild_class`; this file holds the rank table PlayerMobile loads
//! and saves `GuildRank` through: a PlayerData keeps the rank as its index into [`RANKS`].

use muo_server::World;

bitflags::bitflags! {
    /// `RankFlags`.
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct RankFlags: i32 {
        const NONE = 0x0000_0000;
        const CAN_INVITE_PLAYER = 0x0000_0001;
        const ACCESS_GUILD_ITEMS = 0x0000_0002;
        const REMOVE_LOWEST_RANK = 0x0000_0004;
        const REMOVE_PLAYERS = 0x0000_0008;
        const CAN_PROMOTE_DEMOTE = 0x0000_0010;
        const CONTROL_WAR_STATUS = 0x0000_0020;
        const ALLIANCE_CONTROL = 0x0000_0040;
        const CAN_SET_GUILD_TITLE = 0x0000_0080;
        const VOTE = 0x0000_0100;
        const ALL = Self::CAN_INVITE_PLAYER.bits() | Self::REMOVE_PLAYERS.bits() | Self::CAN_PROMOTE_DEMOTE.bits()
            | Self::CONTROL_WAR_STATUS.bits() | Self::ALLIANCE_CONTROL.bits() | Self::CAN_SET_GUILD_TITLE.bits()
            | Self::VOTE.bits() | Self::REMOVE_LOWEST_RANK.bits() | Self::ACCESS_GUILD_ITEMS.bits();
        const MEMBER = Self::REMOVE_LOWEST_RANK.bits() | Self::ACCESS_GUILD_ITEMS.bits() | Self::VOTE.bits();
    }
}

/// `RankDefinition`: `Name` is a cliloc number (`TextDefinition`).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RankDefinition {
    pub name: i32,
    pub rank: i32,
    pub flags: RankFlags,
}

impl RankDefinition {
    pub fn get_flag(&self, flag: RankFlags) -> bool {
        self.flags.intersects(flag)
    }
}

/// `RankDefinition.Ranks`.
pub const RANKS: [RankDefinition; 5] = [
    RankDefinition {
        name: 1062963,
        rank: 0,
        flags: RankFlags::NONE,
    }, // Ronin
    RankDefinition {
        name: 1062962,
        rank: 1,
        flags: RankFlags::MEMBER,
    }, // Member
    RankDefinition {
        name: 1062961, // Emmissary
        rank: 2,
        flags: RankFlags::MEMBER
            .union(RankFlags::REMOVE_PLAYERS)
            .union(RankFlags::CAN_INVITE_PLAYER)
            .union(RankFlags::CAN_SET_GUILD_TITLE)
            .union(RankFlags::CAN_PROMOTE_DEMOTE),
    },
    RankDefinition {
        name: 1062960, // Warlord
        rank: 3,
        flags: RankFlags::MEMBER.union(RankFlags::CONTROL_WAR_STATUS),
    },
    RankDefinition {
        name: 1062959,
        rank: 4,
        flags: RankFlags::ALL,
    }, // Leader
];

pub const LEADER: RankDefinition = RANKS[4];
pub const MEMBER: RankDefinition = RANKS[1];
pub const LOWEST: RankDefinition = RANKS[0];

/// Indexes into [`RANKS`] (what a `PlayerData` stores).
pub const LOWEST_INDEX: i32 = 0;
pub const MEMBER_INDEX: i32 = 1;
pub const LEADER_INDEX: i32 = 4;

/// `Guild.InactiveTime`.
pub const INACTIVE_TIME: muo_primitives::TimeSpan = muo_primitives::TimeSpan::from_days(30);

/// `Guild.NewGuildSystem`: `Core.SE`.
pub fn new_guild_system(w: &World) -> bool {
    crate::core::se(w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_flags_have_the_expected_values() {
        // Member = RemoveLowestRank | AccessGuildItems | CanVote
        assert_eq!(RankFlags::MEMBER.bits(), 0x4 | 0x2 | 0x100);
        assert_eq!(RankFlags::ALL.bits(), 0x1ff);
        assert!(
            LEADER.get_flag(RankFlags::ALLIANCE_CONTROL)
                && !MEMBER.get_flag(RankFlags::REMOVE_PLAYERS)
        );
    }
}
