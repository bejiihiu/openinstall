import { NextRequest, NextResponse } from 'next/server';
import { connectDB } from '@/lib/mongodb';
import Manifest from '@/models/Manifest';

export async function GET(request: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  await connectDB();
  const { id } = await params;
  const manifest = await Manifest.findById(id);
  if (!manifest) return NextResponse.json({ error: 'Not found' }, { status: 404 });
  return NextResponse.json(manifest);
}

export async function DELETE(request: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  await connectDB();
  const { id } = await params;
  // TODO: verify admin role
  await Manifest.findByIdAndDelete(id);
  return NextResponse.json({ success: true });
}
