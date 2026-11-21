CREATE FUNCTION kcna_websearch_query(search_text text)
RETURNS tsquery
LANGUAGE sql
STABLE
PARALLEL SAFE
STRICT
RETURN websearch_to_tsquery('simple'::regconfig, search_text);
