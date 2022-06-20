CREATE TABLE sessions (
    token uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    subject uuid NOT NULL references creators(id),
    created timestamp with time zone NOT NULL DEFAULT current_timestamp
)
