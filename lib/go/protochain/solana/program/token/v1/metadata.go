// Package token_v1 provides off-chain metadata types for Metaplex Token Metadata.
//
// Covers all five token standards:
//   - 0 – NonFungible
//   - 1 – FungibleAsset
//   - 2 – Fungible
//   - 3 – NonFungibleEdition
//   - 4 – ProgrammableNonFungible
//
// See: https://www.metaplex.com/docs/smart-contracts/token-metadata/token-standard
package token_v1

import (
	"encoding/json"
	"fmt"
)

// MetadataAttributeDisplayType controls how wallets / marketplaces render a
// numeric attribute. An empty string means the default string rendering.
//
// See: https://docs.opensea.io/docs/metadata-standards
type MetadataAttributeDisplayType string

const (
	DisplayTypeNumber          MetadataAttributeDisplayType = "number"
	DisplayTypeBoostNumber     MetadataAttributeDisplayType = "boost_number"
	DisplayTypeBoostPercentage MetadataAttributeDisplayType = "boost_percentage"
	DisplayTypeDate            MetadataAttributeDisplayType = "date"
)

// MetadataCategory describes the primary media type of an asset.
type MetadataCategory string

const (
	CategoryImage MetadataCategory = "image"
	CategoryVideo MetadataCategory = "video"
	CategoryAudio MetadataCategory = "audio"
	CategoryVR    MetadataCategory = "vr"
	CategoryHTML  MetadataCategory = "html"
)

// AttributeValue holds the polymorphic value of a MetadataAttribute.
// The underlying type is one of: string, float64, or bool.
type AttributeValue struct {
	raw any
}

// StringValue returns the value as a string and true, or ("", false).
func (v AttributeValue) StringValue() (string, bool) {
	s, ok := v.raw.(string)
	return s, ok
}

// Float64Value returns the value as a float64 and true, or (0, false).
func (v AttributeValue) Float64Value() (float64, bool) {
	f, ok := v.raw.(float64)
	return f, ok
}

// BoolValue returns the value as a bool and true, or (false, false).
func (v AttributeValue) BoolValue() (bool, bool) {
	b, ok := v.raw.(bool)
	return b, ok
}

// Raw returns the underlying value (string | float64 | bool).
func (v AttributeValue) Raw() any { return v.raw }

// String implements fmt.Stringer.
func (v AttributeValue) String() string { return fmt.Sprintf("%v", v.raw) }

// MarshalJSON implements json.Marshaler.
func (v AttributeValue) MarshalJSON() ([]byte, error) {
	return json.Marshal(v.raw)
}

// UnmarshalJSON implements json.Unmarshaler.
// Accepts JSON strings, numbers, and booleans.
func (v *AttributeValue) UnmarshalJSON(data []byte) error {
	var raw any
	if err := json.Unmarshal(data, &raw); err != nil {
		return err
	}
	switch raw.(type) {
	case string, float64, bool:
		v.raw = raw
	case nil:
		v.raw = ""
	default:
		return fmt.Errorf("unsupported attribute value type: %T", raw)
	}
	return nil
}

// MetadataAttribute represents a single trait / attribute of an asset.
//
// Value is intentionally polymorphic:
//   - string  → rendered as a tag / pill
//   - float64 → rendered as bar / numeric (depends on DisplayType)
//   - bool    → rendered as a toggle / checkbox
//
// When DisplayType is "date", Value must be a Unix timestamp (seconds).
type MetadataAttribute struct {
	TraitType string         `json:"trait_type"`
	Value     AttributeValue `json:"value"`
	// DisplayType controls how wallets / marketplaces render the value.
	DisplayType MetadataAttributeDisplayType `json:"display_type,omitempty"`
	// MaxValue is the upper bound for numeric traits (progress bars).
	MaxValue *float64 `json:"max_value,omitempty"`
}

// MetadataFile is a file entry in the asset's file manifest.
type MetadataFile struct {
	// URI is the fully-qualified location (IPFS, Arweave, HTTPS).
	URI string `json:"uri"`
	// Type is the MIME type (e.g. "image/png", "video/mp4").
	Type string `json:"type"`
	// CDN indicates the URI is served through a Content Delivery Network.
	CDN *bool `json:"cdn,omitempty"`
}

// MetadataCreator is the deprecated off-chain creator entry inside
// properties.creators. Canonical creator / royalty data lives on-chain
// in the Metadata account.
type MetadataCreator struct {
	// Address is the Base-58 encoded public key of the creator.
	Address string `json:"address"`
	// Share is the percentage of royalties (0-100, must sum to 100).
	Share int `json:"share"`
}

// MetadataProperties holds the properties bag attached to FungibleAsset,
// NonFungible, and ProgrammableNonFungible metadata.
type MetadataProperties struct {
	// Files is the multi-file manifest for the asset's media.
	Files []MetadataFile `json:"files,omitempty"`
	// Category is the primary media category.
	Category MetadataCategory `json:"category,omitempty"`
	// Deprecated: Creators — on-chain creators are authoritative.
	// Retained for backwards-compatibility with existing off-chain JSON.
	Creators []MetadataCreator `json:"creators,omitempty"`
}

// MetadataCollection is the deprecated off-chain collection grouping.
// Canonical collection data lives on-chain via the Collection field in
// the Metadata account.
type MetadataCollection struct {
	// Name is the human-readable name of the collection.
	Name string `json:"name,omitempty"`
	// Family is the broad family grouping (e.g. "Solana Monkey Business").
	Family string `json:"family,omitempty"`
}

// UniversalTokenMetadata is the universal Metaplex off-chain metadata schema.
//
// It is a superset of all five token standards:
//   - Fungible uses: Name, Symbol, Description, Image
//   - FungibleAsset / NonFungible / NonFungibleEdition / ProgrammableNonFungible
//     add: AnimationURL, ExternalURL, Attributes, Properties, and more
//
// Fields marked deprecated are retained because real-world JSON files still
// contain them. Canonical data for those fields lives on-chain.
//
// See: https://www.metaplex.com/docs/smart-contracts/token-metadata/token-standard
type UniversalTokenMetadata struct {
	// Name is the display name of the asset.
	Name string `json:"name"`
	// Symbol is the ticker symbol (e.g. "USDC"). Required for Fungible tokens.
	Symbol string `json:"symbol,omitempty"`
	// Description is the human-readable description. Supports markdown.
	Description string `json:"description"`
	// Image is the URI to the asset's primary image.
	// For NFTs with an AnimationURL this acts as the thumbnail / poster.
	Image string `json:"image"`
	// AnimationURL is the URI to the rich-media asset (mp4, glb, html, mp3, etc.).
	AnimationURL string `json:"animation_url,omitempty"`
	// ExternalURL is a link to an external page for the project / issuer.
	ExternalURL string `json:"external_url,omitempty"`
	// BackgroundColor is a six-character hex colour (no "#" prefix) used as the
	// background behind the image on marketplaces like OpenSea.
	BackgroundColor string `json:"background_color,omitempty"`
	// Attributes is the array of traits used for rarity, filtering, and display.
	Attributes []MetadataAttribute `json:"attributes,omitempty"`
	// Properties holds the file manifest, media category, and deprecated creators.
	Properties *MetadataProperties `json:"properties,omitempty"`
	// Deprecated: Collection — on-chain Collection field is authoritative.
	// Retained for backwards-compatibility.
	Collection *MetadataCollection `json:"collection,omitempty"`
	// Deprecated: SellerFeeBasisPoints — royalties in basis points (100 = 1%).
	// Canonical value lives on-chain. Retained because many existing JSON files
	// still include it.
	SellerFeeBasisPoints *int `json:"seller_fee_basis_points,omitempty"`
}

// ParseUniversalTokenMetadata unmarshals raw JSON bytes into a
// UniversalTokenMetadata. It returns an error if the JSON is malformed.
func ParseUniversalTokenMetadata(data []byte) (*UniversalTokenMetadata, error) {
	var m UniversalTokenMetadata
	if err := json.Unmarshal(data, &m); err != nil {
		return nil, fmt.Errorf("parsing token metadata: %w", err)
	}
	return &m, nil
}
