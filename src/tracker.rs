use std::fmt;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Tracker {
    Generic,
    ProTracker,
    NoiseTracker,
    FastTracker,
    TakeTracker,
    Startrekker,
    Falcon,
    Oktalyzer,
    UltimateSoundTracker,

    // Used for further heuristics later
    FastOrNoiseTracker,
}

impl fmt::Display for Tracker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted = match self {
            Tracker::Generic => "Generic",
            Tracker::ProTracker => "ProTracker",
            Tracker::TakeTracker => "TakeTracker",
            Tracker::FastTracker => "FastTracker",
            Tracker::NoiseTracker => "NoiseTracker",
            Tracker::Startrekker => "Startrekker",
            Tracker::Falcon => "Falcon",
            Tracker::Oktalyzer => "Oktalyzer",
            Tracker::UltimateSoundTracker => "Ultimate SoundTracker",
            Tracker::FastOrNoiseTracker => "FastTracker/NoiseTracker/ProTracker",
        };
        write!(f, "{}", formatted)
    }
}

// Flattened period tables for ProTracker and finetuned ProTracker
const PROTRACKER_PERIODS: [u16; 7 * 12] = [
    3424, 3232, 3048, 2880, 2712, 2560, 2416, 2280, 2152, 2032, 1920, 1812, 1712, 1616, 1524, 1440,
    1356, 1280, 1208, 1140, 1076, 1016, 960, 906, 856, 808, 762, 720, 678, 640, 604, 570, 538, 508,
    480, 453, 428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240, 226, 214, 202, 190, 180, 170,
    160, 151, 143, 135, 127, 120, 113, 107, 101, 95, 90, 85, 80, 75, 71, 67, 63, 60, 56, 53, 50,
    47, 45, 42, 40, 37, 35, 33, 31, 30, 28,
];

const PROTRACKER_TUNED_PERIODS: [u16; 16 * 12] = [
    1712, 1616, 1524, 1440, 1356, 1280, 1208, 1140, 1076, 1016, 960, 907, 1700, 1604, 1514, 1430,
    1348, 1274, 1202, 1134, 1070, 1010, 954, 900, 1688, 1592, 1504, 1418, 1340, 1264, 1194, 1126,
    1064, 1004, 948, 894, 1676, 1582, 1492, 1408, 1330, 1256, 1184, 1118, 1056, 996, 940, 888,
    1664, 1570, 1482, 1398, 1320, 1246, 1176, 1110, 1048, 990, 934, 882, 1652, 1558, 1472, 1388,
    1310, 1238, 1168, 1102, 1040, 982, 926, 874, 1640, 1548, 1460, 1378, 1302, 1228, 1160, 1094,
    1032, 974, 920, 868, 1628, 1536, 1450, 1368, 1292, 1220, 1150, 1086, 1026, 968, 914, 862, 1814,
    1712, 1616, 1524, 1440, 1356, 1280, 1208, 1140, 1076, 1016, 960, 1800, 1700, 1604, 1514, 1430,
    1350, 1272, 1202, 1134, 1070, 1010, 954, 1788, 1688, 1592, 1504, 1418, 1340, 1264, 1194, 1126,
    1064, 1004, 948, 1774, 1676, 1582, 1492, 1408, 1330, 1256, 1184, 1118, 1056, 996, 940, 1762,
    1664, 1570, 1482, 1398, 1320, 1246, 1176, 1110, 1048, 988, 934, 1750, 1652, 1558, 1472, 1388,
    1310, 1238, 1168, 1102, 1040, 982, 926, 1736, 1640, 1548, 1460, 1378, 1302, 1228, 1160, 1094,
    1032, 974, 920, 1724, 1628, 1536, 1450, 1368, 1292, 1220, 1150, 1086, 1026, 968, 914,
];

pub fn protracker_period_to_note(period: u16, finetune: i8) -> Option<String> {
    if period == 0 {
        return None;
    }

    // Choose the correct period table based on the finetune value
    let table: &[u16] = if finetune == 0 {
        &PROTRACKER_PERIODS
    } else {
        &PROTRACKER_TUNED_PERIODS
    };

    let mut closest_period = u16::MAX;
    let mut closest_index = 0;

    // Iterate over the table to find the closest period
    for (i, &p) in table.iter().enumerate() {
        if (p as i32 - period as i32).abs() < (closest_period as i32 - period as i32).abs() {
            closest_period = p;
            closest_index = i;
        }
    }

    let note_names = [
        "C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-",
    ];
    let note_index = closest_index % 12;
    let octave = closest_index / 12 + 2;

    Some(format!("{}{}", note_names[note_index], octave))
}
