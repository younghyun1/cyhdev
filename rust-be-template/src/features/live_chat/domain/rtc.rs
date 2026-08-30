//! Persistence-independent WebRTC signaling values.

use super::actor::ChatActor;

/// Kind of media track used for publisher bookkeeping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaKind {
    Audio,
    Video,
}

/// One trickled ICE candidate.
#[derive(Debug, Clone)]
pub struct RtcIceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u16>,
}

/// Whether a peer-state update marks presence or departure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RtcPeerPhase {
    Joined,
    Left,
}

/// One participant in a room roster.
#[derive(Debug, Clone)]
pub struct RtcParticipant {
    pub actor: ChatActor,
    pub mic_on: bool,
    pub cam_on: bool,
}

/// Signaling command accepted from a connected client.
#[derive(Debug, Clone)]
pub enum RtcClientSignal {
    Join {
        sdp: String,
        want_audio: bool,
        want_video: bool,
    },
    Answer {
        sdp: String,
    },
    Ice(RtcIceCandidate),
    Leave,
    MediaState {
        mic_on: bool,
        cam_on: bool,
    },
}

/// Signaling event emitted to a connected client.
#[derive(Debug, Clone)]
pub enum RtcServerSignal {
    Answer {
        sdp: String,
    },
    Offer {
        sdp: String,
    },
    Ice(RtcIceCandidate),
    PeerState {
        actor: ChatActor,
        phase: RtcPeerPhase,
        mic_on: bool,
        cam_on: bool,
    },
    Roster {
        participants: Vec<RtcParticipant>,
    },
    Error {
        code: String,
        message: String,
    },
}
