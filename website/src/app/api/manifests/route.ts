import { NextRequest, NextResponse } from 'next/server';
import { connectDB } from '@/lib/mongodb';
import Manifest from '@/models/Manifest';
import User from '@/models/User';

export async function GET() {
  await connectDB();
  const manifests = await Manifest.find().sort({ createdAt: -1 }).limit(50);
  return NextResponse.json(manifests);
}

export async function POST(request: NextRequest) {
  try {
    await connectDB();
    const apiKey = request.headers.get('authorization')?.replace('Bearer ', '');
    if (!apiKey) return NextResponse.json({ error: 'API key required' }, { status: 401 });

    const user = await User.findOne({ apiKey }).select('-password');
    if (!user) return NextResponse.json({ error: 'Invalid API key' }, { status: 401 });

    const body = await request.json();
    const { name, publisher, version, description, homepage, license, packages, sha256, signature } = body;

    if (!name || !publisher || !version || !description) {
      return NextResponse.json({ error: 'Missing required fields' }, { status: 400 });
    }

    const manifest = await Manifest.create({
      name, publisher, version, description, homepage, license, packages, sha256, signature,
      submittedBy: user._id,
    });

    return NextResponse.json(manifest, { status: 201 });
  } catch (error) {
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}
