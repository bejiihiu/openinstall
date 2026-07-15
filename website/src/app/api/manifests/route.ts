import { NextRequest, NextResponse } from 'next/server';
import { connectDB } from '@/lib/mongodb';
import Manifest from '@/models/Manifest';

export async function GET() {
  await connectDB();
  const manifests = await Manifest.find().sort({ createdAt: -1 }).limit(50);
  return NextResponse.json(manifests);
}

export async function POST(request: NextRequest) {
  await connectDB();
  const apiKey = request.headers.get('authorization')?.replace('Bearer ', '');
  if (!apiKey) return NextResponse.json({ error: 'API key required' }, { status: 401 });

  // TODO: validate API key against User collection
  const body = await request.json();
  const manifest = await Manifest.create(body);
  return NextResponse.json(manifest, { status: 201 });
}
