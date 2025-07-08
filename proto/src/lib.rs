//! # Proto
//!
//! Protocol buffer definitions and gRPC service interfaces for Raft.
//!
//! This module contains the generated protobuf code and service definitions
//! for inter-node communication in the Raft cluster.

// Try to include the generated protobuf code, or provide manual definitions
#[cfg(feature = "generated-proto")]
pub mod raft {
    tonic::include_proto!("raft");
}

// Manual protobuf definitions for when protoc is not available
#[cfg(not(feature = "generated-proto"))]
pub mod raft {
    use tonic::{Request, Response, Status};

    // Manual message definitions
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RequestVoteRequest {
        #[prost(uint64, tag = "1")]
        pub term: u64,
        #[prost(string, tag = "2")]
        pub candidate_id: ::prost::alloc::string::String,
        #[prost(uint64, tag = "3")]
        pub last_log_index: u64,
        #[prost(uint64, tag = "4")]
        pub last_log_term: u64,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RequestVoteResponse {
        #[prost(uint64, tag = "1")]
        pub term: u64,
        #[prost(bool, tag = "2")]
        pub vote_granted: bool,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct AppendEntriesRequest {
        #[prost(uint64, tag = "1")]
        pub term: u64,
        #[prost(string, tag = "2")]
        pub leader_id: ::prost::alloc::string::String,
        #[prost(uint64, tag = "3")]
        pub prev_log_index: u64,
        #[prost(uint64, tag = "4")]
        pub prev_log_term: u64,
        #[prost(message, repeated, tag = "5")]
        pub entries: ::prost::alloc::vec::Vec<LogEntry>,
        #[prost(uint64, tag = "6")]
        pub leader_commit: u64,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct AppendEntriesResponse {
        #[prost(uint64, tag = "1")]
        pub term: u64,
        #[prost(bool, tag = "2")]
        pub success: bool,
        #[prost(uint64, tag = "3")]
        pub conflict_index: u64,
        #[prost(uint64, tag = "4")]
        pub conflict_term: u64,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct LogEntry {
        #[prost(uint64, tag = "1")]
        pub index: u64,
        #[prost(uint64, tag = "2")]
        pub term: u64,
        #[prost(int32, tag = "3")]
        pub entry_type: i32,
        #[prost(bytes = "vec", tag = "4")]
        pub data: ::prost::alloc::vec::Vec<u8>,
        #[prost(string, tag = "5")]
        pub client_id: ::prost::alloc::string::String,
        #[prost(uint64, tag = "6")]
        pub sequence_number: u64,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct InstallSnapshotRequest {
        #[prost(uint64, tag = "1")]
        pub term: u64,
        #[prost(string, tag = "2")]
        pub leader_id: ::prost::alloc::string::String,
        #[prost(uint64, tag = "3")]
        pub last_included_index: u64,
        #[prost(uint64, tag = "4")]
        pub last_included_term: u64,
        #[prost(uint64, tag = "5")]
        pub offset: u64,
        #[prost(bytes = "vec", tag = "6")]
        pub data: ::prost::alloc::vec::Vec<u8>,
        #[prost(bool, tag = "7")]
        pub done: bool,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct InstallSnapshotResponse {
        #[prost(uint64, tag = "1")]
        pub term: u64,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct SubmitCommandRequest {
        #[prost(bytes = "vec", tag = "1")]
        pub command: ::prost::alloc::vec::Vec<u8>,
        #[prost(string, tag = "2")]
        pub client_id: ::prost::alloc::string::String,
        #[prost(uint64, tag = "3")]
        pub sequence_number: u64,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct SubmitCommandResponse {
        #[prost(bool, tag = "2")]
        pub success: bool,
        #[prost(string, tag = "3")]
        pub error: ::prost::alloc::string::String,
        #[prost(bytes = "vec", tag = "4")]
        pub result: ::prost::alloc::vec::Vec<u8>,
        #[prost(string, tag = "5")]
        pub leader_id: ::prost::alloc::string::String,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetStatusRequest {}

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct GetStatusResponse {
        #[prost(int32, tag = "1")]
        pub state: i32,
        #[prost(uint64, tag = "2")]
        pub current_term: u64,
        #[prost(string, tag = "3")]
        pub node_id: ::prost::alloc::string::String,
        #[prost(string, tag = "4")]
        pub leader_id: ::prost::alloc::string::String,
        #[prost(uint64, tag = "5")]
        pub commit_index: u64,
        #[prost(uint64, tag = "6")]
        pub last_applied: u64,
        #[prost(uint64, tag = "7")]
        pub log_length: u64,
        #[prost(string, repeated, tag = "8")]
        pub peers: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum NodeState {
        Follower = 0,
        Candidate = 1,
        Leader = 2,
    }

    // Service trait definitions
    #[tonic::async_trait]
    pub trait RaftService: Send + Sync + 'static {
        async fn request_vote(
            &self,
            request: Request<RequestVoteRequest>,
        ) -> Result<Response<RequestVoteResponse>, Status>;

        async fn append_entries(
            &self,
            request: Request<AppendEntriesRequest>,
        ) -> Result<Response<AppendEntriesResponse>, Status>;

        async fn install_snapshot(
            &self,
            request: Request<InstallSnapshotRequest>,
        ) -> Result<Response<InstallSnapshotResponse>, Status>;

        async fn submit_command(
            &self,
            request: Request<SubmitCommandRequest>,
        ) -> Result<Response<SubmitCommandResponse>, Status>;

        async fn get_status(
            &self,
            request: Request<GetStatusRequest>,
        ) -> Result<Response<GetStatusResponse>, Status>;
    }

    // Server and client stubs
    pub struct RaftServiceServer<T> {
        inner: T,
    }

    impl<T> RaftServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self { inner }
        }
    }

    pub struct RaftServiceClient<T> {
        inner: T,
    }
}

pub use raft::*;
