use bitflags::bitflags;


#[derive(Debug)]
pub enum HangingDisposition {
    Both,
    Beneficial,
    Harmful
}


#[derive(Debug)]
pub enum DuelPhase {
    Starting = 0,
    PrePlanning = 1,
    Planning = 2,
    PreExecution = 3,
    Execution = 4,
    Resolution = 5,
    Victory = 6,
    Ended = 7,
    Max = 10
}


#[derive(Debug)]
pub enum SigilInitiativeSwitchMode {
    None,
    Reroll,
    Switch
}

#[derive(Debug)]
pub enum DuelExecutionOrder {
    Sequential,
    Alternating
}


#[derive(Debug)]
pub enum PipAcquiredByEnum {
    Unknown = 0,
    Normal = 1,
    Power = 2,
    NormalToPowerConversion = 4,
    ImpedePips = 5
}


#[derive(Debug)]
pub enum DelayOrder {
    AnyOrder,
    First,
    Second
}


bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct WindowStyle: i32 {
        const HAS_BACK = 1;
        const SCALE_CHILDREN = 2;
        const CAN_MOVE = 4;
        const CAN_SCROLL = 16;
        const CAN_DOCK = 32;
        const FOCUS_LOCKED = 64;
        const CAN_FOCUS = 128;
        const DO_NOT_CAPTURE_MOUSE = 256;
        const IS_TRANSPARENT = 256;
        const EFFECT_FADEID = 512;
        const EFFECT_HIGHLIGHT = 1024;
        const HAS_NO_BORDER = 2048;
        const IGNORE_PARENT_SCALE = 4096;
        const USE_ALPHA_BOUNDS = 8192;
        const AUTO_GROW = 16384;
        const AUTO_SHRINK = 32768;
        const AUTO_RESIZE = 49152;
    }
}



bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct WindowFlags: i32 {
        const VISIBLE = 1;
        const NOCLIP = 2;
        const DOCK_LEFT = 128;
        const DOCK_RIGHT = 256;
        const DOCK_TOP = 512;
        const DOCK_BOTTOM = 1024;
        const HCENTER = 32768;
        const VCENTER = 65536;
        const DOCK_OUTSIDE = 131072;
        const PARENT_WIDTH = 262144;
        const PARENT_HEIGHT = 524288;
        const PARENT_SIZE = 786432;
        const DISABLED = -2147483648;
    }
}

#[derive(Debug)]
pub enum SpellSourceType {
    Caster,
    Pet,
    ShadowCreature,
    Weapon,
    Equipment
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellEffects {
    InvalidSpellEffect = 0,
    Damage = 1,
    DamageNoCrit = 2,
    Heal = 3,
    HealPercent = 4,
    SetHealPercent = 113,
    StealHealth = 5,
    ReduceOverTime = 6,
    DetonateOverTime = 7,
    PushCharm = 8,
    StealCharm = 9,
    PushWard = 10,
    StealWard = 11,
    PushOverTime = 12,
    StealOverTime = 13,
    RemoveCharm = 14,
    RemoveWard = 15,
    RemoveOverTime = 16,
    RemoveAura = 17,
    SwapAll = 18,
    SwapCharm = 19,
    SwapWard = 20,
    SwapOverTime = 21,
    ModifyIncomingDamage = 22,
    ModifyIncomingDamageFlat = 119,
    MaximumIncomingDamage = 23,
    ModifyIncomingHeal = 24,
    ModifyIncomingHealFlat = 118,
    ModifyIncomingDamageType = 25,
    ModifyIncomingArmorPiercing = 26,
    ModifyOutgoingDamage = 27,
    ModifyOutgoingDamageFlat = 121,
    ModifyOutgoingHeal = 28,
    ModifyOutgoingHealFlat = 120,
    ModifyOutgoingDamageType = 29,
    ModifyOutgoingArmorPiercing = 30,
    ModifyOutgoingStealHealth = 31,
    ModifyIncomingStealHealth = 32,
    BounceNext = 33,
    BouncePrevious = 34,
    BounceBack = 35,
    BounceAll = 36,
    AbsorbDamage = 37,
    AbsorbHeal = 38,
    ModifyAccuracy = 39,
    Dispel = 40,
    Confusion = 41,
    CloakedCharm = 42,
    CloakedWard = 43,
    StunResist = 44,
    Clue = 111,
    PipConversion = 45,
    CritBoost = 46,
    CritBlock = 47,
    Polymorph = 48,
    DelayCast = 49,
    ModifyCardCloak = 50,
    ModifyCardDamage = 51,
    ModifyCardAccuracy = 53,
    ModifyCardMutation = 54,
    ModifyCardRank = 55,
    ModifyCardArmorPiercing = 56,
    SummonCreature = 65,
    TeleportPlayer = 66,
    Stun = 67,
    Dampen = 68,
    Reshuffle = 69,
    MindControl = 70,
    ModifyPips = 71,
    ModifyPowerPips = 72,
    ModifyShadowPips = 73,
    ModifyHate = 74,
    DamageOverTime = 75,
    HealOverTime = 76,
    ModifyPowerPipChance = 77,
    ModifyRank = 78,
    StunBlock = 79,
    RevealCloak = 80,
    InstantKill = 81,
    Afterlife = 82,
    DeferredDamage = 83,
    DamagePerTotalPipPower = 84,
    ModifyCardHeal = 52,
    ModifyCardCharm = 57,
    ModifyCardWard = 58,
    ModifyCardOutgoingDamage = 59,
    ModifyCardOutgoingAccuracy = 60,
    ModifyCardOutgoingHeal = 61,
    ModifyCardOutgoingArmorPiercing = 62,
    ModifyCardIncomingDamage = 63,
    ModifyCardAbsorbDamage = 64,
    CloakedWardNoRemove = 86,
    AddCombatTriggerList = 87,
    RemoveCombatTriggerList = 88,
    BacklashDamage = 89,
    ModifyBacklash = 90,
    Intercept = 91,
    ShadowSelf = 92,
    ShadowCreature = 93,
    ModifyShadowCreatureLevel = 94,
    SelectShadowCreatureAttackTarget = 95,
    ShadowDecrementTurn = 96,
    CritBoostSchoolSpecific = 97,
    SpawnCreature = 98,
    UnPolymorph = 99,
    PowerPipConversion = 100,
    ProtectCardBeneficial = 101,
    ProtectCardHarmful = 102,
    ProtectBeneficial = 103,
    ProtectHarmful = 104,
    DivideDamage = 105,
    CollectEssence = 106,
    KillCreature = 107,
    DispelBlock = 108,
    ConfusionBlock = 109,
    ModifyPipRoundRate = 110,
    MaxHealthDamage = 112,
    Untargetable = 114,
    MakeTargetable = 115,
    ForceTargetable = 116,
    RemoveStunBlock = 117,
    ExitCombat = 122,
    SuspendPips = 123,
    ResumePips = 124,
    AutoPass = 125,
    StopAutoPass = 126,
    Vanish = 127,
    StopVanish = 128,
    MaxHealthHeal = 129,
    HealByWard = 130,
    Taunt = 131,
    Pacify = 132,
    RemoveTargetRestriction = 133,
    ConvertHangingEffect = 134,
    AddSpellToDeck = 135,
    AddSpellToHand = 136,
    ModifyIncomingDamageOverTime = 137,
    ModifyIncomingHealOverTime = 138,
    ModifyCardDamageByRank = 139,
    PushConvertedCharm = 140,
    StealConvertedCharm = 141,
    PushConvertedWard = 142,
    StealConvertedWard = 143,
    PushConvertedOverTime = 144,
    StealConvertedOverTime = 145,
    RemoveConvertedCharm = 146,
    RemoveConvertedWard = 147,
    RemoveConvertedOverTime = 148,
    ModifyOverTimeDuration = 149,
    ModifySchoolPips = 150,
}


pub enum EffectTarget {
    InvalidTarget = 0,
    Spell = 1,
    SpecificSpells = 2,
    TargetGlobal = 3,
    EnemyTeam = 4,
    EnemyTeamAllAtOnce = 5,
    FriendlyTeam = 6,
    FriendlyTeamAllAtOnce = 7,
    EnemySingle = 8,
    FriendlySingle = 9,
    Minion = 10,
    FriendlyMinion = 17,
    SelfTarget = 11,
    AtLeastOneEnemy = 13,
    PreselectedEnemySingle = 12,
    MultiTargetEnemy = 14,
    MultiTargetFriendly = 15,
    FriendlySingleNotMe = 16
}


#[derive(Debug)]
pub enum ObjectType {
    Undefined = 0,
    Player = 1,
    Npc = 2,
    Prop = 3,
    Object = 4,
    House = 5,
    Key = 6,
    OldKey = 7,
    Deed = 8,
    Mail = 9,
    Recipe = 17,
    EquipHead = 10,
    EquipChest = 11,
    EquipLegs = 12,
    EquipHands = 13,
    EquipFinger = 14,
    EquipFeet = 15,
    EquipEar = 16,
    BuildingBlock = 18,
    BuildingBlockSolid = 19,
    Golf = 20,
    Door = 21,
    Pet = 22,
    Fabric = 23,
    Window = 24,
    Roof = 25,
    Horse = 26,
    Structure = 27,
    HousingTexture = 28,
    Plant = 29,
}


#[derive(Debug)]
pub enum MagicSchool {
    Ice = 72777,
    Sun = 78483,
    Life = 2330892,
    Fire = 2343174,
    Star = 2625203,
    Myth = 2448141,
    Moon = 2504141,
    Death = 78318724,
    Storm = 83375795,
    Gardening = 663550619,
    CastleMagic = 806477568,
    WhirlyBurly = 931528087,
    Balance = 1027491821,
    Shadow = 1429009101,
    Fishing = 1488274711,
    Cantrips = 1760873841
}


#[derive(Debug)]
enum FogMode {
    Fog = 1,
    Filter = 2
}


bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AccountPermissions: u16 {
        const NO_PERMISSIONS      = 0b0000_0000_0000_0000;
        const CAN_CHAT           = 0b0000_0000_0000_0001;
        const CAN_FILTERED_CHAT  = 0b0000_0000_0000_0010;
        const CAN_OPEN_CHAT      = 0b0000_0000_0000_0100;
        const CAN_OPEN_CHAT_LEGACY = 0b0000_0000_0000_1000;
        const CAN_TRUE_FRIEND_CODE = 0b0000_0000_0001_0000;
        const CAN_GIFT           = 0b0000_0000_0010_0000;
        const CAN_REPORT_BUGS    = 0b0000_0000_0100_0000;
        const UNKNOWN            = 0b0000_0000_1000_0000;
        const UNKNOWN1           = 0b0000_0001_0000_0000;
        const UNKNOWN2           = 0b0000_0010_0000_0000;
        const CAN_EARN_CROWNS_OFFERS = 0b0000_0100_0000_0000;
        const CAN_EARN_CROWNS_BUTTON = 0b0000_1000_0000_0000;
        const UNKNOWN3           = 0b0001_0000_0000_0000;
        const UNKNOWN4           = 0b0010_0000_0000_0000;
        // 5 and 6 are probably not used
        const UNKNOWN5           = 0b0100_0000_0000_0000;
        const UNKNOWN6           = 0b1000_0000_0000_0000;
    }
}


#[derive(Debug)]
pub enum HangingEffectType {
    Any,
    Ward,
    Charm,
    OverTime,
    Specific
}


#[derive(Debug)]
pub enum OutputEffectSelector {
    All,
    MatchedSelectRank
}


#[derive(Debug)]
pub enum CountBasedType {
    SpellKills,
    SpellCrits
}


#[derive(Debug)]
pub enum Operator {
    AND,
    OR
}


#[derive(Debug)]
pub enum RequirementTarget {
    IsMinion,
    HasMinion,
    OnTeam,
    OnOtherTeam,
    OnAnyTeam
}


#[derive(Debug)]
pub enum StatusEffect {
    Stunned,
    Confused
}