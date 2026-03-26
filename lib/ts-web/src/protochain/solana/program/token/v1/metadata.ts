/**
 * Off-chain metadata types for Metaplex Token Metadata.
 *
 * Covers all five token standards:
 *   0 – NonFungible
 *   1 – FungibleAsset
 *   2 – Fungible
 *   3 – NonFungibleEdition
 *   4 – ProgrammableNonFungible
 *
 * @see {@link https://www.metaplex.com/docs/smart-contracts/token-metadata/token-standard}
 */

// ---------------------------------------------------------------------------
// Attributes
// ---------------------------------------------------------------------------

/**
 * Display types that instruct wallets / marketplaces how to render a numeric
 * attribute. Omitting display_type defaults to a plain string trait.
 *
 * @see {@link https://docs.opensea.io/docs/metadata-standards}
 */
export type MetadataAttributeDisplayType = 'number' | 'boost_number' | 'boost_percentage' | 'date';

/**
 * A single trait / attribute of an asset.
 *
 * `value` is intentionally broad:
 *   - string  → rendered as a tag / pill
 *   - number  → rendered as bar / numeric (depends on display_type)
 *   - boolean → rendered as a toggle / checkbox
 *
 * When `display_type` is `"date"`, `value` must be a Unix timestamp (seconds).
 */
export interface MetadataAttribute {
  trait_type: string;
  value: string | number | boolean;
  /** Controls how wallets / marketplaces render the value. */
  display_type?: MetadataAttributeDisplayType;
  /** Upper bound for numeric traits (used by marketplaces for progress bars). */
  max_value?: number;
}

// ---------------------------------------------------------------------------
// Files & creators inside `properties`
// ---------------------------------------------------------------------------

/** Known media categories for the `properties.category` field. */
export type MetadataCategory = 'image' | 'video' | 'audio' | 'vr' | 'html';

/**
 * A file entry in the asset's file manifest.
 *
 * The manifest lets wallets select the best representation
 * (e.g. thumbnail vs. full-res, video vs. still).
 */
export interface MetadataFile {
  /** Fully-qualified URI (IPFS, Arweave, HTTPS). */
  uri: string;
  /** MIME type (e.g. "image/png", "video/mp4", "model/gltf-binary"). */
  type: string;
  /** If true, the URI is served through a CDN. */
  cdn?: boolean;
}

/**
 * Off-chain creator entry inside `properties.creators`.
 *
 * NOTE: This is the **deprecated** off-chain representation.
 * Canonical creator / royalty data lives on-chain in the Metadata account.
 * Included here because many existing assets still carry it in their JSON.
 */
export interface MetadataCreator {
  /** Base-58 encoded public key of the creator. */
  address: string;
  /** Percentage share of royalties (0-100, must sum to 100 across creators). */
  share: number;
}

/**
 * The `properties` bag attached to FungibleAsset, NonFungible, and
 * ProgrammableNonFungible metadata.
 */
export interface MetadataProperties {
  /** Multi-file manifest for the asset's media. */
  files?: MetadataFile[];
  /** Primary media category – drives how wallets render the asset. */
  category?: MetadataCategory;
  /**
   * @deprecated On-chain creators are authoritative. Included for
   * backwards-compatibility with existing off-chain JSON.
   */
  creators?: MetadataCreator[];
}

// ---------------------------------------------------------------------------
// Collection (off-chain, deprecated)
// ---------------------------------------------------------------------------

/**
 * Off-chain collection grouping.
 *
 * NOTE: This is the **deprecated** off-chain convention. Canonical collection
 * data lives on-chain via the Collection field in the Metadata account.
 * Included here because many existing assets still carry it in their JSON.
 */
export interface MetadataCollection {
  /** Human-readable name of the collection. */
  name?: string;
  /** Broad family grouping (e.g. "Solana Monkey Business"). */
  family?: string;
}

// ---------------------------------------------------------------------------
// Universal metadata schema
// ---------------------------------------------------------------------------

/**
 * The universal Metaplex off-chain metadata schema.
 *
 * Designed to be a superset of all five token standards:
 *   - **Fungible** uses: name, symbol, description, image
 *   - **FungibleAsset / NonFungible / NonFungibleEdition / ProgrammableNonFungible**
 *     add: animation_url, external_url, attributes, properties, and more
 *
 * Fields marked deprecated are retained because real-world JSON files still
 * contain them. Canonical data for those fields lives on-chain.
 *
 * @see {@link https://www.metaplex.com/docs/smart-contracts/token-metadata/token-standard}
 */
export interface UniversalTokenMetadata {
  /** The display name of the asset. */
  name: string;

  /** Ticker symbol (e.g. "USDC"). Required for Fungible tokens. */
  symbol?: string;

  /** Human-readable description. Supports markdown. */
  description: string;

  /**
   * URI to the asset's primary image.
   * For NFTs with an animation_url this acts as the thumbnail / poster.
   */
  image: string;

  /** URI to the rich-media asset (mp4, glb, html, mp3, etc.). */
  animation_url?: string;

  /** URL to an external page for the project / issuer. */
  external_url?: string;

  /**
   * Six-character hex colour (no "#" prefix) used as the background
   * behind the image on marketplaces like OpenSea.
   */
  background_color?: string;

  /** Array of traits used for rarity, filtering, and display. */
  attributes?: MetadataAttribute[];

  /** File manifest, media category, and (deprecated) creator list. */
  properties?: MetadataProperties;

  /**
   * @deprecated Off-chain collection grouping. On-chain Collection field is
   * authoritative. Retained for backwards-compatibility.
   */
  collection?: MetadataCollection;

  /**
   * @deprecated Royalties in basis points (100 = 1%). Canonical value lives
   * on-chain in the Metadata account's `seller_fee_basis_points` field.
   * Retained because many existing JSON files still include it.
   */
  seller_fee_basis_points?: number;
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

export class MetadataParser {
  /**
   * Parses raw JSON into a typed {@link UniversalTokenMetadata} object.
   *
   * Performs defensive type-coercion and null-coalescing so that malformed
   * third-party metadata does not cause runtime crashes.
   *
   * @param json - The raw JSON object fetched from a Token Metadata URI.
   * @returns A sanitised UniversalTokenMetadata.
   */
  static fromJSON(json: unknown): UniversalTokenMetadata {
    if (json == null || typeof json !== 'object') {
      throw new Error('MetadataParser: input must be a non-null object.');
    }

    const j = json as Record<string, unknown>;

    return {
      name: asString(j.name, ''),
      symbol: optString(j.symbol),
      description: asString(j.description, ''),
      image: asString(j.image, ''),
      animation_url: optString(j.animation_url),
      external_url: optString(j.external_url),
      background_color: optString(j.background_color),

      attributes: parseAttributes(j.attributes),
      properties: parseProperties(j.properties),
      collection: parseCollection(j.collection),
      seller_fee_basis_points: optNumber(j.seller_fee_basis_points),
    };
  }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

function asString(v: unknown, fallback: string): string {
  return v != null ? String(v) : fallback;
}

function optString(v: unknown): string | undefined {
  return v != null ? String(v) : undefined;
}

function optNumber(v: unknown): number | undefined {
  if (v == null) return undefined;
  const n = Number(v);
  return Number.isFinite(n) ? n : undefined;
}

function parseAttributes(v: unknown): MetadataAttribute[] | undefined {
  if (!Array.isArray(v)) return undefined;

  return v
    .filter((a): a is Record<string, unknown> => a != null && typeof a === 'object')
    .map((a) => {
      const attr: MetadataAttribute = {
        trait_type: asString(a.trait_type, ''),
        value: parseAttributeValue(a.value),
      };
      if (a.display_type != null) {
        attr.display_type = String(a.display_type) as MetadataAttributeDisplayType;
      }
      if (a.max_value != null) {
        const n = Number(a.max_value);
        if (Number.isFinite(n)) attr.max_value = n;
      }
      return attr;
    });
}

function parseAttributeValue(v: unknown): string | number | boolean {
  if (typeof v === 'number' || typeof v === 'boolean') return v;
  return v != null ? String(v) : '';
}

function parseProperties(v: unknown): MetadataProperties | undefined {
  if (v == null || typeof v !== 'object') return undefined;
  const p = v as Record<string, unknown>;

  const props: MetadataProperties = {};

  if (Array.isArray(p.files)) {
    props.files = p.files
      .filter((f): f is Record<string, unknown> => f != null && typeof f === 'object')
      .map((f) => {
        const file: MetadataFile = {
          uri: asString(f.uri, ''),
          type: asString(f.type, 'unknown'),
        };
        if (f.cdn != null) file.cdn = Boolean(f.cdn);
        return file;
      });
  }

  if (p.category != null) {
    props.category = String(p.category) as MetadataCategory;
  }

  if (Array.isArray(p.creators)) {
    props.creators = p.creators
      .filter((c): c is Record<string, unknown> => c != null && typeof c === 'object')
      .map((c) => ({
        address: asString(c.address, ''),
        share: Number.isFinite(Number(c.share)) ? Number(c.share) : 0,
      }));
  }

  return props;
}

function parseCollection(v: unknown): MetadataCollection | undefined {
  if (v == null || typeof v !== 'object') return undefined;
  const c = v as Record<string, unknown>;

  const col: MetadataCollection = {};
  if (c.name != null) col.name = String(c.name);
  if (c.family != null) col.family = String(c.family);

  return col.name != null || col.family != null ? col : undefined;
}
