package token_v1

import (
	"encoding/json"
	"testing"
)

// ---------------------------------------------------------------------------
// Fixtures – taken verbatim from the Metaplex Token Standard documentation
// ---------------------------------------------------------------------------

const fungibleUSDC = `{
  "name": "USD Coin",
  "symbol": "USDC",
  "description": "Fully reserved fiat-backed stablecoin created by Circle.",
  "image": "https://www.circle.com/hs-fs/hubfs/sundaes/USDC.png?width=540&height=540&name=USDC.png"
}`

const fungibleAssetSword = `{
  "name": "SolanaGame Steel Sword",
  "symbol": "SG-SS-1",
  "description": "SolanaGame steel sword available after Level 4",
  "image": "https://arweave.net/26YdhY_eAzv26YdhY1uu9uiA3nmDZYwP8MwZAultcE?ext=jpeg",
  "animation_url": "https://arweave.net/ZAultcE_eAzv26YdhY1uu9uiA3nmDZYwP8MwuiA3nm?ext=glb",
  "external_url": "https://SolanaGame.io",
  "attributes": [
    { "trait_type": "attack", "value": "4" },
    { "trait_type": "defense", "value": "3" },
    { "trait_type": "durability", "value": "47" },
    { "trait_type": "components", "value": "iron: 10; carbon: 1; wood: 2" }
  ]
}`

const nftWithDeprecatedFields = `{
  "name": "SolanaArtProject #1",
  "description": "Generative art on Solana.",
  "image": "https://arweave.net/26YdhY_eAzv26YdhY1uu9uiA3nmDZYwP8MwZAultcE?ext=jpeg",
  "animation_url": "https://arweave.net/ZAultcE_eAzv26YdhY1uu9uiA3nmDZYwP8MwuiA3nm?ext=glb",
  "external_url": "https://example.com",
  "attributes": [
    { "trait_type": "trait1", "value": "value1" },
    { "trait_type": "trait2", "value": "value2" }
  ],
  "properties": {
    "files": [
      { "uri": "https://www.arweave.net/abcd5678?ext=png", "type": "image/png" },
      { "uri": "https://watch.videodelivery.net/9876jkl", "type": "unknown", "cdn": true },
      { "uri": "https://www.arweave.net/efgh1234?ext=mp4", "type": "video/mp4" }
    ],
    "category": "video",
    "creators": [
      { "address": "xEtQ9Fpv62qdc1GYfpNReMasVTe9YW5bHJwfVKqo72u", "share": 100 }
    ]
  }
}`

const nftClean = `{
  "name": "SolanaArtProject #1",
  "description": "Generative art on Solana.",
  "image": "https://arweave.net/26YdhY_eAzv26YdhY1uu9uiA3nmDZYwP8MwZAultcE?ext=jpeg",
  "animation_url": "https://arweave.net/ZAultcE_eAzv26YdhY1uu9uiA3nmDZYwP8MwuiA3nm?ext=glb",
  "external_url": "https://example.com",
  "attributes": [
    { "trait_type": "trait1", "value": "value1" },
    { "trait_type": "trait2", "value": "value2" }
  ],
  "properties": {
    "files": [
      { "uri": "https://www.arweave.net/abcd5678?ext=png", "type": "image/png" },
      { "uri": "https://watch.videodelivery.net/9876jkl", "type": "unknown", "cdn": true },
      { "uri": "https://www.arweave.net/efgh1234?ext=mp4", "type": "video/mp4" }
    ],
    "category": "video"
  }
}`

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

func mustParse(t *testing.T, data string) *UniversalTokenMetadata {
	t.Helper()
	m, err := ParseUniversalTokenMetadata([]byte(data))
	if err != nil {
		t.Fatalf("ParseUniversalTokenMetadata failed: %v", err)
	}
	return m
}

func ptrBool(v bool) *bool          { return &v }
func ptrFloat64(v float64) *float64 { return &v }
func ptrInt(v int) *int             { return &v }

// ---------------------------------------------------------------------------
// Tests – Fungible (USDC)
// ---------------------------------------------------------------------------

func TestParseFungibleUSDC(t *testing.T) {
	m := mustParse(t, fungibleUSDC)

	assertEqual(t, "name", m.Name, "USD Coin")
	assertEqual(t, "symbol", m.Symbol, "USDC")
	assertEqual(t, "description", m.Description, "Fully reserved fiat-backed stablecoin created by Circle.")
	assertEqual(t, "image", m.Image, "https://www.circle.com/hs-fs/hubfs/sundaes/USDC.png?width=540&height=540&name=USDC.png")
}

func TestParseFungibleUSDC_OptionalFieldsAbsent(t *testing.T) {
	m := mustParse(t, fungibleUSDC)

	assertEqual(t, "animation_url", m.AnimationURL, "")
	assertEqual(t, "external_url", m.ExternalURL, "")
	assertEqual(t, "background_color", m.BackgroundColor, "")
	assertNil(t, "attributes", m.Attributes)
	assertNil(t, "properties", m.Properties)
	assertNil(t, "collection", m.Collection)
	assertNil(t, "seller_fee_basis_points", m.SellerFeeBasisPoints)
}

// ---------------------------------------------------------------------------
// Tests – FungibleAsset (Steel Sword)
// ---------------------------------------------------------------------------

func TestParseFungibleAssetSword_TopLevelFields(t *testing.T) {
	m := mustParse(t, fungibleAssetSword)

	assertEqual(t, "name", m.Name, "SolanaGame Steel Sword")
	assertEqual(t, "symbol", m.Symbol, "SG-SS-1")
	assertEqual(t, "description", m.Description, "SolanaGame steel sword available after Level 4")
	assertEqual(t, "image", m.Image, "https://arweave.net/26YdhY_eAzv26YdhY1uu9uiA3nmDZYwP8MwZAultcE?ext=jpeg")
	assertEqual(t, "animation_url", m.AnimationURL, "https://arweave.net/ZAultcE_eAzv26YdhY1uu9uiA3nmDZYwP8MwuiA3nm?ext=glb")
	assertEqual(t, "external_url", m.ExternalURL, "https://SolanaGame.io")
}

func TestParseFungibleAssetSword_Attributes(t *testing.T) {
	m := mustParse(t, fungibleAssetSword)

	if len(m.Attributes) != 4 {
		t.Fatalf("expected 4 attributes, got %d", len(m.Attributes))
	}

	assertAttributeString(t, m.Attributes[0], "attack", "4")
	assertAttributeString(t, m.Attributes[3], "components", "iron: 10; carbon: 1; wood: 2")
}

func TestParseFungibleAssetSword_NoProperties(t *testing.T) {
	m := mustParse(t, fungibleAssetSword)

	assertNil(t, "properties", m.Properties)
	assertNil(t, "collection", m.Collection)
}

// ---------------------------------------------------------------------------
// Tests – NonFungible with deprecated fields
// ---------------------------------------------------------------------------

func TestParseNFTDeprecated_Files(t *testing.T) {
	m := mustParse(t, nftWithDeprecatedFields)

	if m.Properties == nil {
		t.Fatal("expected properties to be present")
	}
	if len(m.Properties.Files) != 3 {
		t.Fatalf("expected 3 files, got %d", len(m.Properties.Files))
	}

	f0 := m.Properties.Files[0]
	assertEqual(t, "files[0].uri", f0.URI, "https://www.arweave.net/abcd5678?ext=png")
	assertEqual(t, "files[0].type", f0.Type, "image/png")
	assertNil(t, "files[0].cdn", f0.CDN)

	f1 := m.Properties.Files[1]
	assertEqual(t, "files[1].uri", f1.URI, "https://watch.videodelivery.net/9876jkl")
	assertEqual(t, "files[1].type", f1.Type, "unknown")
	assertDeepEqual(t, "files[1].cdn", f1.CDN, ptrBool(true))
}

func TestParseNFTDeprecated_Category(t *testing.T) {
	m := mustParse(t, nftWithDeprecatedFields)

	assertEqual(t, "category", string(m.Properties.Category), "video")
}

func TestParseNFTDeprecated_Creators(t *testing.T) {
	m := mustParse(t, nftWithDeprecatedFields)

	if len(m.Properties.Creators) != 1 {
		t.Fatalf("expected 1 creator, got %d", len(m.Properties.Creators))
	}
	assertEqual(t, "creators[0].address", m.Properties.Creators[0].Address, "xEtQ9Fpv62qdc1GYfpNReMasVTe9YW5bHJwfVKqo72u")
	assertEqual(t, "creators[0].share", m.Properties.Creators[0].Share, 100)
}

func TestParseNFTDeprecated_NoTopLevelCollection(t *testing.T) {
	m := mustParse(t, nftWithDeprecatedFields)

	assertNil(t, "collection", m.Collection)
}

// ---------------------------------------------------------------------------
// Tests – NonFungible (clean)
// ---------------------------------------------------------------------------

func TestParseNFTClean_AllFields(t *testing.T) {
	m := mustParse(t, nftClean)

	assertEqual(t, "name", m.Name, "SolanaArtProject #1")
	assertEqual(t, "description", m.Description, "Generative art on Solana.")
	assertEqual(t, "animation_url", m.AnimationURL, "https://arweave.net/ZAultcE_eAzv26YdhY1uu9uiA3nmDZYwP8MwuiA3nm?ext=glb")
	assertEqual(t, "external_url", m.ExternalURL, "https://example.com")
}

func TestParseNFTClean_PropertiesWithoutCreators(t *testing.T) {
	m := mustParse(t, nftClean)

	if m.Properties == nil {
		t.Fatal("expected properties to be present")
	}
	if len(m.Properties.Files) != 3 {
		t.Fatalf("expected 3 files, got %d", len(m.Properties.Files))
	}
	assertEqual(t, "category", string(m.Properties.Category), "video")
	assertNil(t, "creators", m.Properties.Creators)
}

func TestParseNFTClean_SymbolAbsent(t *testing.T) {
	m := mustParse(t, nftClean)

	assertEqual(t, "symbol", m.Symbol, "")
}

// ---------------------------------------------------------------------------
// Tests – Edge cases
// ---------------------------------------------------------------------------

func TestParseInvalidJSON(t *testing.T) {
	_, err := ParseUniversalTokenMetadata([]byte(`not json`))
	if err == nil {
		t.Fatal("expected error for invalid JSON")
	}
}

func TestParseEmptyObject(t *testing.T) {
	m := mustParse(t, `{}`)

	assertEqual(t, "name", m.Name, "")
	assertEqual(t, "description", m.Description, "")
	assertEqual(t, "image", m.Image, "")
}

func TestParseNumericAttributeValue(t *testing.T) {
	m := mustParse(t, `{
		"name": "test", "description": "test", "image": "test",
		"attributes": [{"trait_type": "Level", "value": 5}]
	}`)

	f, ok := m.Attributes[0].Value.Float64Value()
	if !ok {
		t.Fatal("expected float64 value")
	}
	assertEqualFloat(t, "value", f, 5.0)
}

func TestParseBooleanAttributeValue(t *testing.T) {
	m := mustParse(t, `{
		"name": "test", "description": "test", "image": "test",
		"attributes": [{"trait_type": "Legendary", "value": true}]
	}`)

	b, ok := m.Attributes[0].Value.BoolValue()
	if !ok {
		t.Fatal("expected bool value")
	}
	if !b {
		t.Fatal("expected true")
	}
}

func TestParseDisplayTypeAndMaxValue(t *testing.T) {
	m := mustParse(t, `{
		"name": "test", "description": "test", "image": "test",
		"attributes": [
			{"trait_type": "Level", "value": 5, "display_type": "number", "max_value": 10},
			{"trait_type": "Power Boost", "value": 40, "display_type": "boost_percentage"},
			{"trait_type": "Birthday", "value": 1546360800, "display_type": "date"}
		]
	}`)

	if len(m.Attributes) != 3 {
		t.Fatalf("expected 3 attributes, got %d", len(m.Attributes))
	}

	a0 := m.Attributes[0]
	assertEqual(t, "a0.display_type", string(a0.DisplayType), "number")
	assertDeepEqual(t, "a0.max_value", a0.MaxValue, ptrFloat64(10))

	a1 := m.Attributes[1]
	assertEqual(t, "a1.display_type", string(a1.DisplayType), "boost_percentage")
	assertNil(t, "a1.max_value", a1.MaxValue)

	a2 := m.Attributes[2]
	assertEqual(t, "a2.display_type", string(a2.DisplayType), "date")
}

func TestParseTopLevelCollection(t *testing.T) {
	m := mustParse(t, `{
		"name": "test", "description": "test", "image": "test",
		"collection": {"name": "My Collection", "family": "My Family"}
	}`)

	if m.Collection == nil {
		t.Fatal("expected collection to be present")
	}
	assertEqual(t, "collection.name", m.Collection.Name, "My Collection")
	assertEqual(t, "collection.family", m.Collection.Family, "My Family")
}

func TestParseSellerFeeBasisPoints(t *testing.T) {
	m := mustParse(t, `{
		"name": "test", "description": "test", "image": "test",
		"seller_fee_basis_points": 500
	}`)

	assertDeepEqual(t, "seller_fee_basis_points", m.SellerFeeBasisPoints, ptrInt(500))
}

func TestParseBackgroundColor(t *testing.T) {
	m := mustParse(t, `{
		"name": "test", "description": "test", "image": "test",
		"background_color": "FF0000"
	}`)

	assertEqual(t, "background_color", m.BackgroundColor, "FF0000")
}

func TestAttributeValueRoundTrip(t *testing.T) {
	m := mustParse(t, `{
		"name": "test", "description": "test", "image": "test",
		"attributes": [
			{"trait_type": "s", "value": "hello"},
			{"trait_type": "n", "value": 42},
			{"trait_type": "b", "value": false}
		]
	}`)

	data, err := json.Marshal(m)
	if err != nil {
		t.Fatalf("marshal failed: %v", err)
	}

	m2 := mustParse(t, string(data))
	if len(m2.Attributes) != 3 {
		t.Fatalf("expected 3 attributes after round-trip, got %d", len(m2.Attributes))
	}

	s, ok := m2.Attributes[0].Value.StringValue()
	if !ok || s != "hello" {
		t.Fatalf("expected string 'hello', got %v", m2.Attributes[0].Value.Raw())
	}

	f, ok := m2.Attributes[1].Value.Float64Value()
	if !ok || f != 42.0 {
		t.Fatalf("expected float64 42, got %v", m2.Attributes[1].Value.Raw())
	}

	b, ok := m2.Attributes[2].Value.BoolValue()
	if !ok || b {
		t.Fatalf("expected bool false, got %v", m2.Attributes[2].Value.Raw())
	}
}

func TestAttributeValueString(t *testing.T) {
	v := AttributeValue{raw: "hello"}
	assertEqual(t, "String()", v.String(), "hello")

	v2 := AttributeValue{raw: 42.0}
	assertEqual(t, "String()", v2.String(), "42")
}

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

func assertEqual[T comparable](t *testing.T, field string, got, want T) {
	t.Helper()
	if got != want {
		t.Errorf("%s: got %v, want %v", field, got, want)
	}
}

func assertEqualFloat(t *testing.T, field string, got, want float64) {
	t.Helper()
	if got != want {
		t.Errorf("%s: got %v, want %v", field, got, want)
	}
}

func assertNil[T any](t *testing.T, field string, v T) {
	t.Helper()
	if !isNil(v) {
		t.Errorf("%s: expected nil, got %v", field, v)
	}
}

func assertDeepEqual[T any](t *testing.T, field string, got, want T) {
	t.Helper()
	gotJSON, _ := json.Marshal(got)
	wantJSON, _ := json.Marshal(want)
	if string(gotJSON) != string(wantJSON) {
		t.Errorf("%s: got %s, want %s", field, gotJSON, wantJSON)
	}
}

func assertAttributeString(t *testing.T, attr MetadataAttribute, wantTrait, wantValue string) {
	t.Helper()
	assertEqual(t, "trait_type", attr.TraitType, wantTrait)
	s, ok := attr.Value.StringValue()
	if !ok {
		t.Errorf("expected string value for trait %q, got %T", wantTrait, attr.Value.Raw())
		return
	}
	assertEqual(t, "value", s, wantValue)
}

// isNil checks if an interface value is nil, handling typed nils.
func isNil(v any) bool {
	if v == nil {
		return true
	}
	// Use json.Marshal trick: typed nil pointers/slices marshal to "null"
	data, _ := json.Marshal(v)
	return string(data) == "null"
}
