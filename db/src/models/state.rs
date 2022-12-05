/// The following file is a representation, in code, of the taxonomy of bills in U.S. state governments

pub enum BillType {
    /// Most states have these types
    Bill,
    Resolution,

    /// Some states have these types
    JointResolution,
    ConcurrentResolution,
    Memorial,
    JointMemorial,
    ConcurrentMemorial,
    ConstitutionalAmendment,
    Proclamation,
    ExecutiveOrder,
    JointSessionResolution,
    RepealBill,
    Remonstration,
    StudyRequest,
    ConcurrentStudyRequest,

    /// New Hampshire
    Address,
}

struct BillType {
    name: String,
}

struct House;
struct Senate;
// California, Nevada, New Jersey, New York, Wisconsin
struct Assembly;
// Nebraska is unicameral
struct Legislature;

struct Alabama {
    chambers: (House, Senate),
    bill_types: (
        BillType::Bill,
        BillType::Resolution,
        BillType::JointResolution,
    ),
}
