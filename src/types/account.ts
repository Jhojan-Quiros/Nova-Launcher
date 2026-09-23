export type AccountType = "offline" | "microsoft";

export interface MinecraftAccount {
  id: string;
  username: string;
  uuid: string;
  accessToken: string;
  accountType: AccountType;
}

export interface MicrosoftAccountInfo {
  username: string;
  uuid: string;
  tokenExpiresAt: string;
}

export interface DeviceCodeInfo {
  deviceCode: string;
  userCode: string;
  verificationUri: string;
  expiresIn: number;
  interval: number;
}
