//! JSON representation for persistence-independent RTC signaling values.

use std::net::IpAddr;

use serde::{Deserializer, Serializer};
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

use crate::features::live_chat::domain::{
    actor::{ChatActor, ChatActorKey},
    rtc::{
        MediaKind, RtcClientSignal, RtcIceCandidate, RtcParticipant, RtcPeerPhase,
        RtcServerSignal,
    },
};

#[derive(Serialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
enum ChatActorKeyWire<'a> {
    User(&'a Uuid),
    Guest(&'a str),
}

impl serde::Serialize for ChatActorKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wire = match self {
            Self::User(user_id) => ChatActorKeyWire::User(user_id),
            Self::Guest(ip) => ChatActorKeyWire::Guest(ip),
        };
        serde::Serialize::serialize(&wire, serializer)
    }
}

#[derive(Serialize)]
struct ChatActorWire<'a> {
    actor_key: &'a ChatActorKey,
    sender_kind: i16,
    user_id: Option<Uuid>,
    guest_ip: Option<IpAddr>,
    display_name: &'a str,
    country_flag: &'a Option<String>,
    user_profile_picture_url: &'a Option<String>,
}

impl serde::Serialize for ChatActor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(
            &ChatActorWire {
                actor_key: &self.actor_key,
                sender_kind: self.sender_kind,
                user_id: self.user_id,
                guest_ip: self.guest_ip,
                display_name: &self.display_name,
                country_flag: &self.country_flag,
                user_profile_picture_url: &self.user_profile_picture_url,
            },
            serializer,
        )
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum MediaKindWire {
    Audio,
    Video,
}

impl serde::Serialize for MediaKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wire = match self {
            Self::Audio => MediaKindWire::Audio,
            Self::Video => MediaKindWire::Video,
        };
        serde::Serialize::serialize(&wire, serializer)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum RtcPeerPhaseWire {
    Joined,
    Left,
}

impl serde::Serialize for RtcPeerPhase {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wire = match self {
            Self::Joined => RtcPeerPhaseWire::Joined,
            Self::Left => RtcPeerPhaseWire::Left,
        };
        serde::Serialize::serialize(&wire, serializer)
    }
}

#[derive(Deserialize)]
struct RtcIceCandidateInput {
    candidate: String,
    sdp_mid: Option<String>,
    sdp_mline_index: Option<u16>,
}

impl From<RtcIceCandidateInput> for RtcIceCandidate {
    fn from(value: RtcIceCandidateInput) -> Self {
        Self {
            candidate: value.candidate,
            sdp_mid: value.sdp_mid,
            sdp_mline_index: value.sdp_mline_index,
        }
    }
}

#[derive(Serialize)]
struct RtcIceCandidateWire<'a> {
    candidate: &'a str,
    sdp_mid: &'a Option<String>,
    sdp_mline_index: Option<u16>,
}

impl serde::Serialize for RtcIceCandidate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(
            &RtcIceCandidateWire {
                candidate: &self.candidate,
                sdp_mid: &self.sdp_mid,
                sdp_mline_index: self.sdp_mline_index,
            },
            serializer,
        )
    }
}

#[derive(Serialize)]
struct RtcParticipantWire<'a> {
    actor: &'a ChatActor,
    mic_on: bool,
    cam_on: bool,
}

impl serde::Serialize for RtcParticipant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde::Serialize::serialize(
            &RtcParticipantWire {
                actor: &self.actor,
                mic_on: self.mic_on,
                cam_on: self.cam_on,
            },
            serializer,
        )
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RtcClientSignalWire {
    Join {
        sdp: String,
        want_audio: bool,
        want_video: bool,
    },
    Answer {
        sdp: String,
    },
    Ice(RtcIceCandidateInput),
    Leave,
    MediaState {
        mic_on: bool,
        cam_on: bool,
    },
}

impl<'de> serde::Deserialize<'de> for RtcClientSignal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = <RtcClientSignalWire as serde::Deserialize>::deserialize(deserializer)?;
        Ok(match wire {
            RtcClientSignalWire::Join {
                sdp,
                want_audio,
                want_video,
            } => Self::Join {
                sdp,
                want_audio,
                want_video,
            },
            RtcClientSignalWire::Answer { sdp } => Self::Answer { sdp },
            RtcClientSignalWire::Ice(candidate) => Self::Ice(candidate.into()),
            RtcClientSignalWire::Leave => Self::Leave,
            RtcClientSignalWire::MediaState { mic_on, cam_on } => {
                Self::MediaState { mic_on, cam_on }
            }
        })
    }
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RtcServerSignalWire<'a> {
    Answer {
        sdp: &'a str,
    },
    Offer {
        sdp: &'a str,
    },
    Ice(&'a RtcIceCandidate),
    PeerState {
        actor: &'a ChatActor,
        phase: RtcPeerPhase,
        mic_on: bool,
        cam_on: bool,
    },
    Roster {
        participants: &'a [RtcParticipant],
    },
    Error {
        code: &'a str,
        message: &'a str,
    },
}

impl serde::Serialize for RtcServerSignal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let wire = match self {
            Self::Answer { sdp } => RtcServerSignalWire::Answer { sdp },
            Self::Offer { sdp } => RtcServerSignalWire::Offer { sdp },
            Self::Ice(candidate) => RtcServerSignalWire::Ice(candidate),
            Self::PeerState {
                actor,
                phase,
                mic_on,
                cam_on,
            } => RtcServerSignalWire::PeerState {
                actor,
                phase: *phase,
                mic_on: *mic_on,
                cam_on: *cam_on,
            },
            Self::Roster { participants } => RtcServerSignalWire::Roster { participants },
            Self::Error { code, message } => RtcServerSignalWire::Error { code, message },
        };
        serde::Serialize::serialize(&wire, serializer)
    }
}
