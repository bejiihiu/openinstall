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

  const existing = await User.findOne({ email });
  if (existing) {
    return NextResponse.json({ error: 'Email already registered' }, { status: 409 });
  }

  const hashedPassword = await hashPassword(password);
  const user = await User.create({ email, password: hashedPassword });

  return NextResponse.json({ success: true, userId: user._id }, { status: 201 });
}
