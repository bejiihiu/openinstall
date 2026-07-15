import { NextRequest, NextResponse } from 'next/server';
import { connectDB } from '@/lib/mongodb';
import { verifyToken } from '@/lib/auth';

export async function GET(request: NextRequest) {
  try {
    await connectDB();
    const token = request.headers.get('authorization')?.replace('Bearer ', '');
    if (!token) return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });

    let decoded;
    try {
      decoded = verifyToken(token);
    } catch {
      return NextResponse.json({ error: 'Invalid token' }, { status: 401 });
    }

    // TODO: implement history tracking in OpenInstall CLI
    // For now, return empty array
    return NextResponse.json([]);
  } catch {
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}
