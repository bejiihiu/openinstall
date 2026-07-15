import bcrypt from "bcryptjs";
import jwt from "jsonwebtoken";

export class TokenError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'TokenError';
  }
}

function getJwtSecret(): string {
  const secret = process.env.JWT_SECRET;
  if (!secret) {
    throw new Error("Please define the JWT_SECRET environment variable");
  }
  if (secret.length < 32) {
    throw new Error("JWT_SECRET must be at least 32 characters");
  }
  return secret;
}

export async function hashPassword(password: string): Promise<string> {
  return bcrypt.hash(password, 12);
}

export async function comparePassword(
  password: string,
  hashedPassword: string
): Promise<boolean> {
  return bcrypt.compare(password, hashedPassword);
}

export function generateToken(payload: Record<string, unknown>): string {
  return jwt.sign(payload, getJwtSecret(), { expiresIn: "7d" });
}

export function verifyToken(token: string): { userId: string; email: string } {
  try {
    const decoded = jwt.verify(token, getJwtSecret()) as { userId: string; email: string };
    return decoded;
  } catch (err) {
    if (err instanceof jwt.TokenExpiredError) {
      throw new TokenError('Token expired');
    }
    throw new TokenError('Invalid token');
  }
}
