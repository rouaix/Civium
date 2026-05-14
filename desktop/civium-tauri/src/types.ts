export interface IdentityInfo {
  cid_short: string;
  cid_full: string;
  secret_b58: string;
}

export interface NetworkInfo {
  cid_short: string;
  cid_full: string;
  name: string;
  member_count: number;
  is_directory: boolean;
}

export interface MemberInfo {
  cid_short: string;
  display_name: string;
  circle: number;
  role: string;
}

export interface PendingMemberInfo {
  cid_short: string;
  display_name: string;
  requested_at: number;
}

export interface ConnectionInfo {
  peer_cid_short: string;
  peer_name: string;
  state: string;
}

export interface NodeStatus {
  running: boolean;
  listen_addrs: string[];
}

export interface MessageDisplay {
  id: string;
  author_cid_short: string;
  author_name: string;
  body: string;
  sent_at: number;
  is_direct: boolean;
  to_cid_short: string | null;
}

export interface ProposalInfo {
  id: string;
  title: string;
  description: string;
  options: string[];
  created_by: string;
  created_at: number;
  closes_at: number;
  quorum_percent: number;
  status: "open" | "closed" | "cancelled";
}

export interface OptionResult {
  label: string;
  votes: number;
  percent: number;
}

export interface VoteResultInfo {
  proposal_id: string;
  total_votes: number;
  total_members: number;
  participation_percent: number;
  quorum_reached: boolean;
  options: OptionResult[];
  winner: number | null;
}

export interface DelegationInfo {
  delegator_cid_short: string;
  delegate_cid_short: string;
  proposal_id: string | null;
  created_at: number;
}

export interface DirectoryEntryInfo {
  id: string;
  directory_cid_short: string;
  kind: "network" | "member";
  subject_cid_short: string;
  subject_name: string;
  description: string;
  contact_addr: string | null;
  published_by: string;
  published_at: number;
  tags: string[];
  source_dir_name: string | null;
}

export interface FederationInfo {
  id: string;
  host_cid_short: string;
  peer_cid_short: string;
  peer_name: string;
  peer_addr: string | null;
  added_by: string;
  added_at: number;
}

export interface AdminActionInfo {
  id: string;
  kind: string;
  taken_by: string;
  taken_at: number;
  contest_window_secs: number;
  contest_count: number;
  status: "active" | "confirmed" | "suspended" | "reversed" | "upheld";
  suspended_proposal_id: string | null;
}
