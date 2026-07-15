import mongoose, { Schema, Document } from 'mongoose';

export interface IUser extends Document {
  email: string;
  password: string;
  role: 'user' | 'publisher' | 'admin';
  apiKey?: string;
  favorites: mongoose.Types.ObjectId[];
  createdAt: Date;
}

const UserSchema = new Schema<IUser>({
  email: { type: String, required: true, unique: true },
  password: { type: String, required: true },
  role: { type: String, enum: ['user', 'publisher', 'admin'], default: 'user' },
  apiKey: { type: String, unique: true, sparse: true },
  favorites: [{ type: Schema.Types.ObjectId, ref: 'Manifest' }],
}, { timestamps: true });

export default mongoose.models.User || mongoose.model<IUser>('User', UserSchema);
