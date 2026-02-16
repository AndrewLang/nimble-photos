export interface JwtClaims {
  sub: string;
  roles?: string[];
  exp: number;
}
