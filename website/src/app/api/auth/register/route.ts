import { NextRequest, NextResponse } from 'next/server';
import { connectDB } from '@/lib/mongodb';
import { hashPassword } from '@/lib/auth';
import User from '@/models/User';

export async function POST(request: NextRequest) {
  await connectDB();
  const { email, password } = await request.json();

  if (!email || !password) {
    return NextResponse.json({ error: 'Email and password required' }, { status: 400 });
  }

  if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
    return NextResponse.json({ error: 'Invalid email format' }, { status: 400 });
  }

  if (password.length < 8) {
    return NextResponse.json({ error: 'Password must be at least 8 characters' }, { status: 400 });
  }

  const hashedPassword = await hashPassword(password);

  try {
    const user = await User.create({ email, password: hashedPassword });
    return NextResponse.json({ success: true, userId: user._id }, { status: 201 });
  } catch (err) {
    if (err instanceof Error && 'code' in err && err.code === 11000) {
      return NextResponse.json({ error: 'Email already registered' }, { status: 409 });
    }
    throw err;
  }
}
