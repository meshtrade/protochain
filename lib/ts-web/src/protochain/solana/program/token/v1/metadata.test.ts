import { describe, expect, it } from 'vitest';

import { MetadataParser } from './metadata';

// ---------------------------------------------------------------------------
// Fixtures – taken verbatim from the Metaplex Token Standard documentation
// ---------------------------------------------------------------------------

const FUNGIBLE_USDC = {
  name: 'USD Coin',
  symbol: 'USDC',
  description: 'Fully reserved fiat-backed stablecoin created by Circle.',
  image: 'https://www.circle.com/hs-fs/hubfs/sundaes/USDC.png?width=540&height=540&name=USDC.png',
};

const FUNGIBLE_ASSET_SWORD = {
  name: 'SolanaGame Steel Sword',
  symbol: 'SG-SS-1',
  description: 'SolanaGame steel sword available after Level 4',
  image: 'https://arweave.net/26YdhY_eAzv26YdhY1uu9uiA3nmDZYwP8MwZAultcE?ext=jpeg',
  animation_url: 'https://arweave.net/ZAultcE_eAzv26YdhY1uu9uiA3nmDZYwP8MwuiA3nm?ext=glb',
  external_url: 'https://SolanaGame.io',
  attributes: [
    { trait_type: 'attack', value: '4' },
    { trait_type: 'defense', value: '3' },
    { trait_type: 'durability', value: '47' },
    { trait_type: 'components', value: 'iron: 10; carbon: 1; wood: 2' },
  ],
};

const NFT_WITH_DEPRECATED_FIELDS = {
  name: 'SolanaArtProject #1',
  description: 'Generative art on Solana.',
  image: 'https://arweave.net/26YdhY_eAzv26YdhY1uu9uiA3nmDZYwP8MwZAultcE?ext=jpeg',
  animation_url: 'https://arweave.net/ZAultcE_eAzv26YdhY1uu9uiA3nmDZYwP8MwuiA3nm?ext=glb',
  external_url: 'https://example.com',
  attributes: [
    { trait_type: 'trait1', value: 'value1' },
    { trait_type: 'trait2', value: 'value2' },
  ],
  properties: {
    files: [
      { uri: 'https://www.arweave.net/abcd5678?ext=png', type: 'image/png' },
      { uri: 'https://watch.videodelivery.net/9876jkl', type: 'unknown', cdn: true },
      { uri: 'https://www.arweave.net/efgh1234?ext=mp4', type: 'video/mp4' },
    ],
    category: 'video',
    collection: { name: 'Solflare X NFT', family: 'Solflare' },
    creators: [{ address: 'xEtQ9Fpv62qdc1GYfpNReMasVTe9YW5bHJwfVKqo72u', share: 100 }],
  },
};

const NFT_CLEAN = {
  name: 'SolanaArtProject #1',
  description: 'Generative art on Solana.',
  image: 'https://arweave.net/26YdhY_eAzv26YdhY1uu9uiA3nmDZYwP8MwZAultcE?ext=jpeg',
  animation_url: 'https://arweave.net/ZAultcE_eAzv26YdhY1uu9uiA3nmDZYwP8MwuiA3nm?ext=glb',
  external_url: 'https://example.com',
  attributes: [
    { trait_type: 'trait1', value: 'value1' },
    { trait_type: 'trait2', value: 'value2' },
  ],
  properties: {
    files: [
      { uri: 'https://www.arweave.net/abcd5678?ext=png', type: 'image/png' },
      { uri: 'https://watch.videodelivery.net/9876jkl', type: 'unknown', cdn: true },
      { uri: 'https://www.arweave.net/efgh1234?ext=mp4', type: 'video/mp4' },
    ],
    category: 'video',
  },
};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('MetadataParser.fromJSON', () => {
  // ---- Fungible (minimal) ------------------------------------------------

  describe('Fungible token (USDC)', () => {
    it('parses required fields', () => {
      const meta = MetadataParser.fromJSON(FUNGIBLE_USDC);

      expect(meta.name).toBe('USD Coin');
      expect(meta.symbol).toBe('USDC');
      expect(meta.description).toBe('Fully reserved fiat-backed stablecoin created by Circle.');
      expect(meta.image).toBe(
        'https://www.circle.com/hs-fs/hubfs/sundaes/USDC.png?width=540&height=540&name=USDC.png',
      );
    });

    it('leaves optional fields undefined', () => {
      const meta = MetadataParser.fromJSON(FUNGIBLE_USDC);

      expect(meta.animation_url).toBeUndefined();
      expect(meta.external_url).toBeUndefined();
      expect(meta.background_color).toBeUndefined();
      expect(meta.attributes).toBeUndefined();
      expect(meta.properties).toBeUndefined();
      expect(meta.collection).toBeUndefined();
      expect(meta.seller_fee_basis_points).toBeUndefined();
    });
  });

  // ---- FungibleAsset (with attributes) -----------------------------------

  describe('FungibleAsset (Steel Sword)', () => {
    it('parses all top-level fields', () => {
      const meta = MetadataParser.fromJSON(FUNGIBLE_ASSET_SWORD);

      expect(meta.name).toBe('SolanaGame Steel Sword');
      expect(meta.symbol).toBe('SG-SS-1');
      expect(meta.description).toBe('SolanaGame steel sword available after Level 4');
      expect(meta.image).toBe(
        'https://arweave.net/26YdhY_eAzv26YdhY1uu9uiA3nmDZYwP8MwZAultcE?ext=jpeg',
      );
      expect(meta.animation_url).toBe(
        'https://arweave.net/ZAultcE_eAzv26YdhY1uu9uiA3nmDZYwP8MwuiA3nm?ext=glb',
      );
      expect(meta.external_url).toBe('https://SolanaGame.io');
    });

    it('parses string-valued attributes', () => {
      const meta = MetadataParser.fromJSON(FUNGIBLE_ASSET_SWORD);

      expect(meta.attributes).toHaveLength(4);
      expect(meta.attributes![0]).toEqual({ trait_type: 'attack', value: '4' });
      expect(meta.attributes![3]).toEqual({
        trait_type: 'components',
        value: 'iron: 10; carbon: 1; wood: 2',
      });
    });

    it('leaves properties and collection undefined when absent', () => {
      const meta = MetadataParser.fromJSON(FUNGIBLE_ASSET_SWORD);

      expect(meta.properties).toBeUndefined();
      expect(meta.collection).toBeUndefined();
    });
  });

  // ---- NonFungible with deprecated fields --------------------------------

  describe('NonFungible with deprecated collection & creators', () => {
    it('parses properties.files with cdn flag', () => {
      const meta = MetadataParser.fromJSON(NFT_WITH_DEPRECATED_FIELDS);

      expect(meta.properties).toBeDefined();
      expect(meta.properties!.files).toHaveLength(3);
      expect(meta.properties!.files![0]).toEqual({
        uri: 'https://www.arweave.net/abcd5678?ext=png',
        type: 'image/png',
      });
      expect(meta.properties!.files![1]).toEqual({
        uri: 'https://watch.videodelivery.net/9876jkl',
        type: 'unknown',
        cdn: true,
      });
    });

    it('parses properties.category', () => {
      const meta = MetadataParser.fromJSON(NFT_WITH_DEPRECATED_FIELDS);

      expect(meta.properties!.category).toBe('video');
    });

    it('parses deprecated properties.creators', () => {
      const meta = MetadataParser.fromJSON(NFT_WITH_DEPRECATED_FIELDS);

      expect(meta.properties!.creators).toHaveLength(1);
      expect(meta.properties!.creators![0]).toEqual({
        address: 'xEtQ9Fpv62qdc1GYfpNReMasVTe9YW5bHJwfVKqo72u',
        share: 100,
      });
    });

    it('ignores unknown nested keys (collection inside properties)', () => {
      // The `collection` key sits inside `properties` in the fixture – it is
      // not a recognised field of MetadataProperties, so the parser should
      // silently drop it. The top-level `collection` should be undefined.
      const meta = MetadataParser.fromJSON(NFT_WITH_DEPRECATED_FIELDS);

      expect(meta.collection).toBeUndefined();
    });
  });

  // ---- NonFungible without deprecated fields -----------------------------

  describe('NonFungible (clean, no deprecated fields)', () => {
    it('parses all fields correctly', () => {
      const meta = MetadataParser.fromJSON(NFT_CLEAN);

      expect(meta.name).toBe('SolanaArtProject #1');
      expect(meta.description).toBe('Generative art on Solana.');
      expect(meta.animation_url).toBe(
        'https://arweave.net/ZAultcE_eAzv26YdhY1uu9uiA3nmDZYwP8MwuiA3nm?ext=glb',
      );
      expect(meta.external_url).toBe('https://example.com');
    });

    it('parses properties without creators', () => {
      const meta = MetadataParser.fromJSON(NFT_CLEAN);

      expect(meta.properties).toBeDefined();
      expect(meta.properties!.files).toHaveLength(3);
      expect(meta.properties!.category).toBe('video');
      expect(meta.properties!.creators).toBeUndefined();
    });

    it('symbol defaults to undefined when absent', () => {
      const meta = MetadataParser.fromJSON(NFT_CLEAN);

      expect(meta.symbol).toBeUndefined();
    });
  });

  // ---- Edge cases & defensive parsing ------------------------------------

  describe('edge cases', () => {
    it('throws on null input', () => {
      expect(() => MetadataParser.fromJSON(null)).toThrow('input must be a non-null object');
    });

    it('throws on non-object input', () => {
      expect(() => MetadataParser.fromJSON('not an object')).toThrow(
        'input must be a non-null object',
      );
    });

    it('coerces missing required fields to empty strings', () => {
      const meta = MetadataParser.fromJSON({});

      expect(meta.name).toBe('');
      expect(meta.description).toBe('');
      expect(meta.image).toBe('');
    });

    it('handles numeric attribute values', () => {
      const meta = MetadataParser.fromJSON({
        name: 'test',
        description: 'test',
        image: 'test',
        attributes: [{ trait_type: 'Level', value: 5 }],
      });

      expect(meta.attributes![0].value).toBe(5);
      expect(typeof meta.attributes![0].value).toBe('number');
    });

    it('handles boolean attribute values', () => {
      const meta = MetadataParser.fromJSON({
        name: 'test',
        description: 'test',
        image: 'test',
        attributes: [{ trait_type: 'Legendary', value: true }],
      });

      expect(meta.attributes![0].value).toBe(true);
      expect(typeof meta.attributes![0].value).toBe('boolean');
    });

    it('parses display_type and max_value on attributes', () => {
      const meta = MetadataParser.fromJSON({
        name: 'test',
        description: 'test',
        image: 'test',
        attributes: [
          { trait_type: 'Level', value: 5, display_type: 'number', max_value: 10 },
          { trait_type: 'Power Boost', value: 40, display_type: 'boost_percentage' },
          { trait_type: 'Birthday', value: 1546360800, display_type: 'date' },
        ],
      });

      expect(meta.attributes![0]).toEqual({
        trait_type: 'Level',
        value: 5,
        display_type: 'number',
        max_value: 10,
      });
      expect(meta.attributes![1]).toEqual({
        trait_type: 'Power Boost',
        value: 40,
        display_type: 'boost_percentage',
      });
      expect(meta.attributes![2]).toEqual({
        trait_type: 'Birthday',
        value: 1546360800,
        display_type: 'date',
      });
    });

    it('parses top-level collection (name + family)', () => {
      const meta = MetadataParser.fromJSON({
        name: 'test',
        description: 'test',
        image: 'test',
        collection: { name: 'My Collection', family: 'My Family' },
      });

      expect(meta.collection).toEqual({ name: 'My Collection', family: 'My Family' });
    });

    it('parses seller_fee_basis_points', () => {
      const meta = MetadataParser.fromJSON({
        name: 'test',
        description: 'test',
        image: 'test',
        seller_fee_basis_points: 500,
      });

      expect(meta.seller_fee_basis_points).toBe(500);
    });

    it('parses background_color', () => {
      const meta = MetadataParser.fromJSON({
        name: 'test',
        description: 'test',
        image: 'test',
        background_color: 'FF0000',
      });

      expect(meta.background_color).toBe('FF0000');
    });

    it('skips malformed entries in attributes array', () => {
      const meta = MetadataParser.fromJSON({
        name: 'test',
        description: 'test',
        image: 'test',
        attributes: [null, 'invalid', { trait_type: 'valid', value: 'yes' }],
      });

      expect(meta.attributes).toHaveLength(1);
      expect(meta.attributes![0].trait_type).toBe('valid');
    });

    it('skips malformed entries in properties.files array', () => {
      const meta = MetadataParser.fromJSON({
        name: 'test',
        description: 'test',
        image: 'test',
        properties: {
          files: [null, 42, { uri: 'https://example.com/img.png', type: 'image/png' }],
        },
      });

      expect(meta.properties!.files).toHaveLength(1);
      expect(meta.properties!.files![0].uri).toBe('https://example.com/img.png');
    });

    it('ignores non-finite seller_fee_basis_points', () => {
      const meta = MetadataParser.fromJSON({
        name: 'test',
        description: 'test',
        image: 'test',
        seller_fee_basis_points: 'not a number',
      });

      expect(meta.seller_fee_basis_points).toBeUndefined();
    });

    it('returns undefined collection when object has neither name nor family', () => {
      const meta = MetadataParser.fromJSON({
        name: 'test',
        description: 'test',
        image: 'test',
        collection: {},
      });

      expect(meta.collection).toBeUndefined();
    });
  });
});
