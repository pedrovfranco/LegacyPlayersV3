#![allow(clippy::map_entry)]

use crate::modules::data::tools::{RetrieveNPC, RetrieveServer};
use crate::modules::data::Data;
use std::collections::{BTreeSet, HashMap};

pub struct WoWCBTLParser {
    pub server_id: i32,
    pub expansion_id: u8,
    pub start_parse: u64,
    pub end_parse: u64,
    pub timezone: i8, // Can be inferred from the site upload?

    pub active_difficulty: Vec<(u8, u64, u64)>,
    pub active_map: HashMap<u16, (u64, Vec<(u64, u64)>)>,
    pub participation: HashMap<u64, (u64, bool, Vec<(u64, u64)>)>,
    pub found_player: HashMap<u64, (String, u8)>,
    pub retail_player_server: HashMap<u64, (u32, String, Vec<(u64, Vec<(u32, Option<u32>)>)>)>
}

impl WoWCBTLParser {
    pub fn new(data: &Data, server_id: i32, start_parse: u64, end_parse: u64, timezone: i8) -> Self {
        let expansion_id;
        // Retail Classic
        if server_id == -1 {
            expansion_id = 1;
        } else {
            expansion_id = data.get_server(server_id as u32).unwrap().expansion_id;
        }
        WoWCBTLParser {
            server_id,
            expansion_id,
            start_parse,
            end_parse,
            timezone,
            active_difficulty: Vec::new(),
            active_map: HashMap::new(),
            participation: HashMap::new(),
            found_player: HashMap::new(),
            retail_player_server: HashMap::new(),
        }
    }

    pub fn get_active_map(&self, data: &Data, npc_id: u32) -> Option<u16> {
        data.get_npc(self.expansion_id, npc_id).and_then(|npc| npc.map_id)
    }

    pub fn get_difficulty_by_spell_id(&self, spell_id: u32) -> Option<u8> {
        lazy_static! {
            static ref DIFFICULTY_10_NHC: BTreeSet<u32> = [
                // Naxx
                // Crystalsong uses 10 man Disrupting shout in 25 => 29107
                28783, 28786, 28796, 28794, 29484, 28741, 29213, 29214, 29998, 55594, 29204, 55604, 55607, 27831, 27989, 27994, 29317,
                28884, 28882, 57374, 57376, 28883, 28308, 28157, 28135, 28134, 28167, 28531, 28542, 28547, 28478, 28479,
                // Obsidian Sanctum
                // 57570, 57579, 56910, 56908,
                // Eye of Eternity
                // 56272,
                // Vault of Archavon
                58689, 58695, 64213, 64216, 66808, 66809, 66725, 66765, 66665, 66670, 72122, 72120, 72034, 72090
            ].iter().cloned().collect();
            static ref DIFFICULTY_10_HC: BTreeSet<u32> = [
            ].iter().cloned().collect();
            static ref DIFFICULTY_25_NHC: BTreeSet<u32> = [
                // Naxx
                // 55543
                // 56407, 56405, 55609, 55645, 55638
                56090, 54022, 54098, 54125, 54122, 54835, 54836, 55011, 55601, 55052, 55646,
                57467, 57369, 57464, 57465, 57466, 59192, 54364, 54528, 54529, 54531, 55799, 55665, 55699, 55802, 55807,
                // Obsidian Sanctum
                // 59126, 59127, 58957, 58956,
                // Eye of Eternity
                // 60072,
                // Vault of Archavon
                60883, 58692, 65279, 67331, 67333, 68160, 68161, 67328, 67329, 71993, 72004, 72096, 72104
            ].iter().cloned().collect();
            static ref DIFFICULTY_25_HC: BTreeSet<u32> = [
            ].iter().cloned().collect();
        }

        if self.expansion_id < 3 {
            return None; // Difficulty determined by map_id
        }

        if DIFFICULTY_10_NHC.contains(&spell_id) {
            return Some(3);
        } else if DIFFICULTY_10_HC.contains(&spell_id) {
            return Some(5);
        } else if DIFFICULTY_25_NHC.contains(&spell_id) {
            return Some(4);
        } else if DIFFICULTY_25_HC.contains(&spell_id) {
            return Some(6);
        }

        None
    }

    pub fn collect_player(&mut self, unit_id: u64, unit_name: &str, spell_id: u32) {
        lazy_static! {
            static ref WARRIOR_SPELLS: BTreeSet<u32> = [
                // Heroic Strike
                31827,41975,45026,47449,47450,52221,53395,57846,59035,59607,62444,69566,78,284,285,1608,11564,11565,11566,11567,25286,25710,25712,29426,29567,29707,30324,
                // Mortal Strike
                65926,67542,68782,68783,68784,71552,9347,12294,13737,15708,16856,17547,19643,21551,21552,21553,24573,25248,27580,29572,30330,31911,32736,35054,37335,39171,40220,43441,43529,44268,47485,47486,57789,
                // Bloodthirst
                57790,57791,57792,60017,71938,23880,23881,23885,23892,23893,23894,25251,30335,30474,30475,30476,31996,31997,31998,33964,35123,35125,35947,35948,35949,39070,39071,39072,40423,55968,55969,55970,
                // Devastate
                20243,30016,30017,30022,36891,36894,38849,38967,44452,47497,47498,57795,60018,62317,69902
            ].iter().cloned().collect();
            static ref PALADIN_SPELLS: BTreeSet<u32> = [
                // Divinie Illumination
                31842,71166,
                // Holy Light
                // 58053,66112,68011,68012,68013,635,639,647,1026,1042,3472,10328,10329,13952,15493,25263,25292,27135,27136,29383,29427,29562,31713,32769,37979,43451,44479,46029,48781,48782,52444,56539,
                // Flash of Light
                // 19750,19939,19940,19941,19942,19943,25514,27137,33641,37249,37254,37257,48784,48785,57766,59997,66113,66922,68008,68009,68010,71930,
                // Judgement of Light
                // 20185,20267,20271,28775,57774,
                // Judgement of Wisdom
                // 20186,20268,53408,
                // Divine Storm
                53385,54171,54172,58127,66006,
                // Holy Shield
                9800,20925,20927,20928,27179,31904,32777,48951,48952,
                // Crusader Strike
                35395
            ].iter().cloned().collect();
            static ref ROGUE_SPELLS: BTreeSet<u32> = [
                // Sinister Strike
                1752,1757,1758,1759,1760,8621,11293,11294,14873,15581,15667,19472,26861,26862,46558,48637,48638,57640,59409,60195,69920,71145,
                // Slice and Dice
                5171,6434,6774,30470,43547,60847,
                // Eviscerate
                2098,6760,6761,6762,8623,8624,11299,11300,15691,15692,26865,27611,31016,41177,46189,48667,48668,57641,60008,65957,67709,68094,68095,68096,68317,71933,
                // Mutilate
                1329,5374,27576,32319,32320,32321,34411,34412,34413,34414,34415,34416,34417,34418,34419,41103,48661,48662,48663,48664,48665,48666,60850
            ].iter().cloned().collect();
            static ref PRIEST_SPELLS: BTreeSet<u32> = [
                // Flash Heal
                71782,71783,2061,9472,9473,9474,10915,10916,10917,17137,17138,17843,25233,25235,27608,38588,42420,43431,43516,43575,48070,48071,56331,56919,66104,68023,68024,68025,71595,
                // Penance
                69905,69906,71137,71138,71139,47540,47666,47750,47757,47758,52983,52984,52985,52986,52987,52988,52998,52999,53000,53001,53002,53003,53005,53006,53007,54518,54520,66097,66098,68029,68030,68031,
                // Circle of Healing
                34861,34863,34864,34865,34866,41455,48088,48089,49306,61964,
                // Shadow Word: Pain
                48124,48125,57778,59864,60005,60446,589,65541,594,68088,970,68089,992,68090,2767,72318,10892,72319,10893,10894,11639,14032,15654,17146,19776,23268,23952,24212,25367,25368,27605,30854,30898,34441,34941,34942,37275,41355,46560,
                // Mind Flay
                48156,52586,54339,54805,57779,57941,15407,58381,16568,59367,17165,59974,17311,60006,17312,60472,17313,65488,17314,68042,18807,68043,22919,68044,23953,25387,26044,26143,28310,29407,29570,32417,35507,37276,37330,37621,38243,40842,42396,43512,46562,48155
            ].iter().cloned().collect();
            static ref HUNTER_SPELLS: BTreeSet<u32> = [
                // Aimed Shot
                19434,20900,20901,20902,20903,20904,27065,27632,30614,31623,38370,38861,44271,46460,48871,49049,49050,52718,54615,59243,60954,65883,67977,67978,67979,
                // Multi-Shot
                52270,52813,59244,59515,59713,66081,2643,70513,14288,14289,14290,14443,18651,20735,21390,25294,27021,28751,29576,30990,31942,34879,34974,36979,38310,38383,41187,41448,43205,44285,48098,48872,49047,49048,
                // Serpent Sting
                1978,13549,13550,13551,13552,13553,13554,13555,25295,27016,31975,35511,36984,38859,38914,39182,49000,49001,
                // Explosive Shot
                15495,53301,53352,56298,60051,60052,60053,65866,67983,67984,67985,69975,71126,
                // Bestial Wrath
                19574,37587,38371,38484
            ].iter().cloned().collect();
            static ref MAGE_SPELLS: BTreeSet<u32> = [
                // Frostbolt
                41384,21369,61730,41486,23102,61747,42719,23412,62583,42803,24942,62601,25304,63913,42841,27071,65807,116,42842,27072,68003,205,43083,28478,68004,837,43428,28479,68005,7322,44606,29457,69274,8406,44843,29926,69573,8407,46035,29954,70277,8408,46987,30942,70327,9672,49037,31296,71318,10179,50378,50721,31622,71420,10180,10181,54791,32364,72007,11538,55802,32370,72166,12675,55807,32984,72167,12737,56775,34347,72501,13322,56837,35316,72502,13439,57665,36279,15043,57825,36710,15497,58457,36990,15530,58535,37930,16249,59017,38238,38534,16799,59251,38645,17503,59280,38697,20297,59638,38826,20792,59855,39064,20806,61087,40429,20819,61461,40430,20822,61590,
                // Fireball
                23411,62796,15665,38641,24374,63789,16101,38692,24611,63815,16412,38824,25306,66042,16413,39267,16415,40554,27070,68310,16788,40598,133,29456,68926,17290,40877,143,29925,69570,18082,41383,145,29953,69583,18105,41484,3140,30218,69668,18108,42802,8400,30534,70282,18199,42832,8401,30691,70409,18392,42833,8402,30943,70754,18796,42834,9053,30967,71153,19391,42853,9487,31262,71500,31620,71501,19816,44189,9488,10148,32363,71504,20420,44202,10149,32369,71748,20678,44237,10150,32414,71928,20692,45580,10151,32491,72023,20714,45595,10578,33417,72024,20793,45748,11839,33793,72163,20797,46164,11921,33794,72164,20808,46988,11985,34083,20811,47074,12466,34348,20815,49512,13140,34653,20823,52282,21072,54094,13375,36711,21159,54095,13438,36805,21162,54096,14034,36920,21402,57628,15228,36971,21549,59994,15242,37111,22088,61567,15536,37329,23024,61909,15662,37463,
                // Arcane Blast
                50545,51797,51830,56969,58462,59257,10833,59909,16067,65791,18091,67997,20883,67998,22893,67999,22920,22940,24857,30451,30661,31457,32935,34793,35314,35927,36032,37126,38342,38344,38538,38881,40837,40881,42894,42896,42897,49198,
                // Arcane Missiles
                31751,33031,33419,33462,33552,33553,5143,33832,5144,33833,5145,33988,7269,33989,7270,34446,8416,34447,8417,35033,8418,35034,8419,38263,10211,10212,38264,10273,38699,10274,38700,15735,38703,15736,38704,15790,39414,15791,42843,22272,42844,22273,42845,25345,42846,58529,25346,58531,27075,61592,27076,61593,29955,29956,31742,31743
            ].iter().cloned().collect();
            static ref WARLOCK_SPELLS: BTreeSet<u32> = [
                // Shadowbolt
                30505,68151,17434,47076,30686,68152,17435,47248,31618,68153,17483,47808,31627,69028,17509,47809,18111,49084,32666,69068,18138,50455,686,32860,69211,18164,51363,695,33335,69212,18205,51432,705,34344,69387,18211,51608,1088,36714,69577,18214,52257,1106,36868,69972,18217,52534,7617,36972,70043,19728,53086,7619,36986,70080,19729,53333,7641,36987,70182,20298,54113,9613,38378,70208,38386,70270,20791,55984,11659,11660,38628,70386,20807,56405,11661,38825,70387,20816,57374,12471,38892,71143,20825,57464,12739,39025,71254,21077,57644,13440,39026,71296,21141,57725,13480,39297,71297,22336,58827,14106,39309,71936,22677,59016,14122,40185,72008,24668,59246,15232,41069,72503,25307,59254,15472,41280,72504,26006,59351,27209,59357,15537,41957,72901,29317,59389,16408,43330,72960,29487,59575,16409,43649,72961,29626,60015,16410,43667,75330,29640,61558,16783,45055,75331,29927,61562,16784,45679,75384,30055,65821,17393,45680,
                // Immolate
                27215,29928,36637,36638,37668,38805,348,38806,707,41958,1094,44267,2941,44518,8981,46042,9034,46191,9275,47810,9276,47811,11665,75383,11667,11668,11962,11984,12742,15505,15506,15570,15661,15732,15733,17883,18542,20294,20787,20800,20826,25309,
                // Corruption
                47782,47812,47813,56898,57645,58811,172,60016,6222,61563,6223,65810,7648,68133,11671,68134,11672,68135,13530,70602,16402,70904,16985,71937,17510,18088,18376,18656,21068,23642,25311,27216,28829,30938,31405,32063,32197,37113,37961,39212,39621,41988,
                // Unstable Affliction
                30108,30404,30405,31117,34438,34439,35183,43522,43523,47841,47843,65812,65813,68154,68155,68156,68157,68158,68159
            ].iter().cloned().collect();
            static ref SHAMAN_SPELLS: BTreeSet<u32> = [
                // Stormstrike
                17364,32175,32176,51876,64757,65970,65971,65972,
                // Earth Shock
                68101,68102,8042,8044,8045,8046,10412,10413,10414,13281,13728,15501,22885,23114,24685,25025,25454,26194,43305,47071,49230,49231,54511,56506,57783,60011,61668,65973,68100,
                // Lightning Bolt
                48895,22414,68112,49237,23592,68113,49238,25448,68114,49239,25449,69567,26098,69970,49240,31764,71136,403,49418,34345,71934,529,49454,35010,548,51587,36152,915,51618,37273,943,53044,37661,6041,53314,37664,8246,54843,38465,9532,55044,39065,10391,56326,56891,41184,10392,12167,57780,42024,13482,57781,43526,13527,59006,43903,14109,59024,45075,14119,59081,45284,15207,59169,45286,15208,59199,45287,15234,59683,45288,15801,59863,45289,16782,60009,45290,45291,18081,60032,45292,18089,61374,45293,19874,61893,45294,20295,63809,45295,20802,64098,45296,20805,64696,48698,20824,65987,
                // Chain Lightning
                59517,33643,59716,37448,59844,39066,61528,39945,40536,61879,41183,421,62131,42441,930,63479,42804,2860,64213,43435,10605,64215,44318,12058,64390,45297,15117,64758,45298,15305,64759,45299,15659,67529,45300,16006,68319,69696,45301,16033,16921,75362,45302,20831,45868,21179,46380,22355,48140,23106,48699,23206,49268,24680,49269,25021,49270,25439,49271,25442,50830,52383,27567,54334,28167,54531,28293,59082,28900,59220,31330,59223,31717,59273,32337,
                // Healing Wave
                59083,60012,61569,67528,68318,69958,331,71133,332,75382,547,913,939,959,8005,10395,10396,11986,12491,12492,15982,25357,25391,25396,26097,38330,43548,48700,49272,49273,51586,52868,55597,57785,58980,
                // Riptide
                22419,61295,61299,61300,61301,66053,68118,68119,68120,75367
            ].iter().cloned().collect();
            static ref DRUID_SPELLS: BTreeSet<u32> = [
                // Insect Swarm
                5570,24974,24975,24976,24977,27013,48468,65855,67941,67942,67943,
                // Starfire
                2912,8949,8950,8951,9875,9876,21668,25298,26986,35243,38935,40344,48464,48465,65854,67947,67948,67949,75332,
                // Mangle (Cat)
                33876,33982,33983,48565,48566,
                // Mangle (Bear)
                33878,33986,33987,48563,48564,
                // Regrowth
                8936,8938,8939,8940,8941,9750,9856,9857,9858,16561,20665,22373,22695,26980,27637,28744,34361,39000,39125,48442,48443,66067,67968,67969,67970,69882,71141,
                // Rejuvenation
                48440,48441,53607,64801,66065,67971,774,67972,1058,67973,1430,69898,2090,70691,2091,71142,3627,8070,8910,9839,9840,9841,12160,15981,20664,20701,25299,26981,26982,27532,28716,28722,28723,28724,31782,32131,38657,42544
            ].iter().cloned().collect();
            static ref DEATHKNIGHT_SPELLS: BTreeSet<u32> = [
                // Icy Touch
                45477,49723,49896,49903,49904,49909,50349,52372,52378,53549,55313,55331,59011,59131,60952,66021,67718,67881,67938,67939,67940,69916,70589,70591,
                // Frost Strike
                43568,49143,51416,51417,51418,51419,55268,60951,66047,66196,66958,66959,66960,66961,66962,67935,67936,67937,
                // Scourge Strike
                28265,55090,55265,55270,55271,70890,71488,
                // Blood Strike
                45902,49926,49927,49928,49929,49930,52374,52377,59130,60945,61696,66215,66975,66976,66977,66978,66979,
                // Death Strike
                45463,45469,45470,49923,49924,49998,49999,53639,66188,66950,66951,66952,66953,71489,
                // Heart Strike
                55050,55258,55259,55260,55261,55262,55978
            ].iter().cloned().collect();
        }

        if !self.found_player.contains_key(&unit_id) || self.found_player.get(&unit_id).unwrap().1 == 12 {
            let hero_class_id = if WARRIOR_SPELLS.contains(&spell_id) {
                1
            } else if PALADIN_SPELLS.contains(&spell_id) {
                2
            } else if HUNTER_SPELLS.contains(&spell_id) {
                3
            } else if ROGUE_SPELLS.contains(&spell_id) {
                4
            } else if PRIEST_SPELLS.contains(&spell_id) {
                5
            } else if DEATHKNIGHT_SPELLS.contains(&spell_id) {
                6
            } else if SHAMAN_SPELLS.contains(&spell_id) {
                7
            } else if MAGE_SPELLS.contains(&spell_id) {
                8
            } else if WARLOCK_SPELLS.contains(&spell_id) {
                9
            } else if DRUID_SPELLS.contains(&spell_id) {
                11
            } else {
                12
            };

            self.found_player.insert(unit_id, (unit_name.replace("\"", ""), hero_class_id));
        }
    }
}
