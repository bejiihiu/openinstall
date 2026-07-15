import mongoose, { Schema, Document, Model } from "mongoose";

export interface IManifest extends Document {
  name: string;
  publisher: string;
  version: string;
  description: string;
  homepage?: string;
  license?: string;
  packages: Map<string, string>;
  sha256?: string;
  signature?: string;
  submittedBy: mongoose.Types.ObjectId;
  downloads: number;
  createdAt: Date;
  updatedAt: Date;
}

const ManifestSchema = new Schema<IManifest>(
  {
    name: {
      type: String,
      required: true,
      trim: true,
    },
    publisher: {
      type: String,
      required: true,
      trim: true,
    },
    version: {
      type: String,
      required: true,
      trim: true,
    },
    description: {
      type: String,
      required: true,
    },
    homepage: {
      type: String,
    },
    license: {
      type: String,
    },
    packages: {
      type: Map,
      of: String,
    },
    sha256: {
      type: String,
    },
    signature: {
      type: String,
    },
    submittedBy: {
      type: Schema.Types.ObjectId,
      ref: "User",
      required: true,
    },
    downloads: {
      type: Number,
      default: 0,
    },
  },
  {
    timestamps: true,
  }
);

const Manifest: Model<IManifest> =
  mongoose.models.Manifest || mongoose.model<IManifest>("Manifest", ManifestSchema);

export default Manifest;
