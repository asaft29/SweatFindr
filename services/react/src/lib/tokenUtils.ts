interface JwtPayload {
  exp: number;
  iat: number;
  sub: number;
  role: string;
  iss: string;
  jti: string;
}

export function decodeJwt(token: string): JwtPayload | null {
  try {
    const parts = token.split(".");
    if (parts.length !== 3) {
      return null;
    }

    const payload = parts[1];
    const decoded = atob(payload);
    return JSON.parse(decoded) as JwtPayload;
  } catch (error) {
    console.error("Failed to decode JWT:", error);
    return null;
  }
}

export function isTokenExpired(token: string): boolean {
  const payload = decodeJwt(token);
  if (!payload) {
    return true;
  }

  const nowInSeconds = Math.floor(Date.now() / 1000);
  return payload.exp < nowInSeconds;
}

export function getTokenExpirationDate(token: string): Date | null {
  const payload = decodeJwt(token);
  if (!payload) {
    return null;
  }

  return new Date(payload.exp * 1000);
}
