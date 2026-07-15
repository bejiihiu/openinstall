import { NextRequest, NextResponse } from 'next/server';
import { connectDB } from '@/lib/mongodb';
import { verifyToken } from '@/lib/auth';
import User from '@/models/User';

const isValidObjectId = (id: string) => /^[0-9a-fA-F]{24}$/.test(id);

export async function GET(request: NextRequest) {
  await connectDB();
  const token = request.headers.get('authorization')?.replace('Bearer ', '');
  if (!token) return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });

  let decoded;
  try {
    decoded = verifyToken(token);
  } catch {
    return NextResponse.json({ error: 'Invalid token' }, { status: 401 });
  }

  const user = await User.findById(decoded.userId).populate('favorites').select('-password');
  if (!user) return NextResponse.json({ error: 'User not found' }, { status: 404 });

  return NextResponse.json(user.favorites);
}

export async function POST(request: NextRequest) {
  await connectDB();
  const token = request.headers.get('authorization')?.replace('Bearer ', '');
  if (!token) return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });

  let decoded;
  try {
    decoded = verifyToken(token);
  } catch {
    return NextResponse.json({ error: 'Invalid token' }, { status: 401 });
  }

  const { manifestId } = await request.json();
  if (!manifestId || !isValidObjectId(manifestId)) {
    return NextResponse.json({ error: 'Invalid manifest ID' }, { status: 400 });
  }

  const user = await User.findById(decoded.userId);
  if (!user) return NextResponse.json({ error: 'User not found' }, { status: 404 });

  const favStrings = user.favorites.map((id: { toString(): string }) => id.toString());
  if (!favStrings.includes(manifestId)) {
    user.favorites.push(manifestId);
    await user.save();
  }

  return NextResponse.json({ success: true });
}

export async function DELETE(request: NextRequest) {
  await connectDB();
  const token = request.headers.get('authorization')?.replace('Bearer ', '');
  if (!token) return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });

  let decoded;
  try {
    decoded = verifyToken(token);
  } catch {
    return NextResponse.json({ error: 'Invalid token' }, { status: 401 });
  }

  const { manifestId } = await request.json();
  if (!manifestId || !isValidObjectId(manifestId)) {
    return NextResponse.json({ error: 'Invalid manifest ID' }, { status: 400 });
  }

  const user = await User.findById(decoded.userId);
  if (!user) return NextResponse.json({ error: 'User not found' }, { status: 404 });

  user.favorites = user.favorites.filter((id: { toString(): string }) => id.toString() !== manifestId);
  await user.save();

  return NextResponse.json({ success: true });
}
