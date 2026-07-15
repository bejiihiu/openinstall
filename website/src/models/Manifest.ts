import mongoose, { Schema, Document } from 'mongoose';

export interface IManifest extends Document {
  name: string;
  publisher: string;
  version: string;
  description: string;
  homepage?: string;
  license?: string;
  packages: Record<string, string>;
  sha256?: string;
  signature?: string;
  submittedBy: mongoose.Types.ObjectId;
  downloads: number;
  createdAt: Date;
}

const ManifestSchema = new Schema<IManifest>({
  name: { type: String, required: true },
  publisher: { type: String, required: true },
  version: { type: String, required: true },
  description: { type: String, required: true },
  homepage: String,
  license: String,
  packages: { type: Map, of: String },
  sha256: String,
  signature: String,
  submittedBy: { type: Schema.Types.ObjectId, ref: 'User', required: true },
  downloads: { type: Number, default: 0 },
}, { timestamps: true });

export default mongoose.models.Manifest || mongoose.model<IManifest>('Manifest', ManifestSchema);
