interface JwtPayload {
  exp?: number;
}

export const isTokenExpired = (token: string, bufferSeconds = 10): boolean => {
  try {
    const parts = token.split(".");
    if (parts.length !== 3 || !parts[1]) {
      return true;
    }

    const base64 = parts[1].replace(/-/g, "+").replace(/_/g, "/");
    const jsonPayload = decodeURIComponent(
      atob(base64)
        .split("")
        .map(
          (character) =>
            `%${character.charCodeAt(0).toString(16).padStart(2, "0")}`,
        )
        .join(""),
    );
    const payload = JSON.parse(jsonPayload) as JwtPayload;

    if (typeof payload.exp !== "number") {
      return false;
    }

    return payload.exp <= Date.now() / 1000 + bufferSeconds;
  } catch (error: unknown) {
    console.error("Error decoding JWT token:", error);
    return true;
  }
};
