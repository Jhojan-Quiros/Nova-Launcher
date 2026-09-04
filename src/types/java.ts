export interface JavaRuntime {
  id: string;
  name: string;
  path: string;
  majorVersion: number;
  rawVersion: string;
  isValid: boolean;
}