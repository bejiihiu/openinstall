import { NextRequest, NextResponse } from 'next/server';
import { connectDB } from '@/lib/mongodb';
import Manifest from '@/models/Manifest';
import { verifyToken } from '@/lib/auth';
import User from '@/models/User';

const isValidObjectId = (id: string) => /^[0-9a-fA-F]{24}$/.test(id);

export async function GET(request: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  await connectDB();
  const { id } = await params;

  if (!isValidObjectId(id)) {
    return NextResponse.json({ error: 'Invalid manifest ID' }, { status: 400 });
  }

  try {
    const manifest = await Manifest.findById(id);
    if (!manifest) return NextResponse.json({ error: 'Not found' }, { status: 404 });
    return NextResponse.json(manifest);
  } catch {
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}

export async function DELETE(request: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  await connectDB();
  const { id } = await params;

  if (!isValidObjectId(id)) {
    return NextResponse.json({ error: 'Invalid manifest ID' }, { status: 400 });
  }

  const token = request.headers.get('authorization')?.replace('Bearer ', '');
  if (!token) return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });

  let decoded;
  try {
    decoded = verifyToken(token);
  } catch {
    return NextResponse.json({ error: 'Invalid token' }, { status: 401 });
  }

  const user = await User.findById(decoded.userId).select('-password');
  if (!user || user.role !== 'admin') {
    return NextResponse.json({ error: 'Forbidden' }, { status: 403 });
  }

  await Manifest.findByIdAndDelete(id);
  return NextResponse.json({ success: true });
}
