-- The last block the Runes index has fully indexed, written in the same
-- transaction as that block. The ledger only proves the last block that had
-- Rune activity, which trails the indexed tip on a quiet chain.
CREATE TABLE IF NOT EXISTS indexed_chain_tip (
    id              BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (id),
    block_height    NUMERIC NOT NULL,
    block_hash      TEXT NOT NULL
);
