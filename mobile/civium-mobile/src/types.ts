export interface IdentityInfo {
  cid_short:  string;
  cid_full:   string;
  secret_b58: string;
}

export interface NetworkInfo {
  cid_short:    string;
  name:         string;
  member_count: number;
}

export interface MessageInfo {
  id:               string;
  author_cid_short: string;
  body:             string;
  sent_at:          number;
  is_direct:        boolean;
}

export type RootStackParamList = {
  Onboarding: undefined;
  Pairing:    undefined;
  Networks:   { identity: IdentityInfo };
  Messages:   { identity: IdentityInfo; network: NetworkInfo };
};
