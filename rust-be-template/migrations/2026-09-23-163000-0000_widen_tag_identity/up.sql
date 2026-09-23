-- A smallint identity allows 32,767 tags, and every conflicting insert used to
-- consume a value. Widening both sides of the relation keeps the foreign key
-- valid; changing an identity column's type also widens its sequence, whose
-- maximum becomes the integer maximum.
ALTER TABLE public.tags ALTER COLUMN tag_id TYPE integer;
ALTER TABLE public.post_tags ALTER COLUMN tag_id TYPE integer;
