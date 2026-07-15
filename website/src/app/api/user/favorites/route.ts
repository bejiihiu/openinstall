import { NextRequest, NextResponse } from 'next/server';
import { connectDB } from '@/lib/mongodb';
import { verifyToken } from '@/lib/auth';
import User from '@/models/User';

export async function GET(request: NextRequest) {
  await connectDB();
  const token = request.headers.get('authorization')?.replace('Bearer ', '');
  if (!token) return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });

  const decoded = verifyToken(token);
  const user = await User.findById(decoded.userId).populate('favorites');
  if (!user) return NextResponse.json({ error: 'User not found' }, { status: 404 });

  return NextResponse.json(user.favorites);
}

export async function POST(request: NextRequest) {
  await connectDB();
  const token = request.headers.get('authorization')?.replace('Bearer ', '');
  if (!token) return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });

  const decoded = verifyToken(token);
  const { manifestId } = await request.json();

  const user = await User.findById(decoded.userId);
  if (!user) return NextResponse.json({ error: 'User not found' }, { status: 404 });

  if (!user.favorites.includes(manifestId)) {
    user.favorites.push(manifestId);
    await user.save();
  }

  return NextResponse.json({ success: true });
}
