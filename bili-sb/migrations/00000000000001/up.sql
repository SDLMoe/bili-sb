CREATE TABLE videos (
  -- up to 2^51
  aid         BIGINT       NOT NULL PRIMARY KEY,
  -- max size of title is 80 * UTF-16 char size = 160
  title       VARCHAR(160) NOT NULL,
  update_time TIMESTAMP    NOT NULL
);

CREATE TABLE video_parts (
  cid         BIGINT       NOT NULL PRIMARY KEY,
  aid         BIGINT       NOT NULL REFERENCES videos(aid),
  -- not sure for length, assuming the same as video
  title       VARCHAR(160) NOT NULL,
  duration    REAL         NOT NULL
);

CREATE INDEX idx_video_parts_aid ON video_parts(aid);

CREATE TABLE users (
  id            UUID      NOT NULL PRIMARY KEY,
  register_time TIMESTAMP NOT NULL,
  register_ip   CIDR      NOT NULL,
  -- last vote / upload segment timestmap
  last_operation_time TIMESTAMP
);

CREATE TABLE segments (
  id        UUID   NOT NULL PRIMARY KEY,
  cid       BIGINT NOT NULL REFERENCES video_parts(cid),
  start_seg REAL   NOT NULL,
  end_seg   REAL   NOT NULL,
  upvote    INT    NOT NULL,
  downvote  INT    NOT NULL,
  submitter UUID   NOT NULL REFERENCES users(id)
);

CREATE INDEX idx_segments_cid ON segments(cid);
