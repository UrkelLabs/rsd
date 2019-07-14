// total number of buckets for tried addresses
pub(crate) const TRIED_BUCKET_COUNT_LOG2: u32 = 8;

// total number of buckets for new addresses
pub(crate) const NEW_BUCKET_COUNT_LOG2: u32 = 10;

// maximum allowed number of entries in buckets for new and tried addresses
pub(crate) const BUCKET_SIZE_LOG2: u32 = 6;

// over how many buckets entries with tried addresses from a single group (/16 for IPv4) are spread
pub(crate) const TRIED_BUCKETS_PER_GROUP: u32 = 8;

// over how many buckets entries with new addresses originating from a single group are spread
pub(crate) const NEW_BUCKETS_PER_SOURCE_GROUP: u32 = 64;

// in how many buckets for entries with new addresses a single address may occur
const NEW_BUCKETS_PER_ADDRESS: u32 = 8;

// how old addresses can maximally be
pub(crate) const HORIZON_DAYS: u32 = 30;

// after how many failed attempts we give up on a new node
pub(crate) const RETRIES: u32 = 3;

// how many successive failures are allowed ...
pub(crate) const MAX_FAILURES: u32 = 10;

// ... in at least this many days
pub(crate) const MIN_FAIL_DAYS: u32 = 7;

// how recent a successful connection should be before we allow an address to be evicted from tried
const REPLACEMENT_HOUR: u32 = 4;

// the maximum percentage of nodes to return in a getaddr call
const GETADDR_MAX_PERCENT: u32 = 23;

// the maximum number of nodes to return in a getaddr call
const GETADDR_GETADDR_MAX: u32 = 2500;

//TODO docs
//Maybe not crate here, pub(self)?
pub(crate) const TRIED_BUCKET_COUNT: usize = 1 << TRIED_BUCKET_COUNT_LOG2;
pub(crate) const NEW_BUCKET_COUNT: usize = 1 << NEW_BUCKET_COUNT_LOG2;
pub(crate) const BUCKET_SIZE: usize = 1 << BUCKET_SIZE_LOG2;

// the maximum number of tried addr collisions to store
const SET_TRIED_COLLISION_SIZE: u32 = 10;

// the maximum time we'll spend trying to resolve a tried table collision, in seconds
const TEST_WINDOW: u32 = 40 * 60;
