//! October 2026 snapshot of US dollar prices per megajoule for electricity, fuels, and foods.
//!
//! Source retail or wholesale prices are converted with the helpers below. This is not a live feed.

/// Megajoules in one kilowatt-hour.
pub const MJ_PER_KWH: f64 = 3.6;

/// Litres in one US gallon.
pub const LITERS_PER_US_GALLON: f64 = 3.785411784;

/// Megajoules per (food) kilocalorie.
pub const KCAL_TO_MJ: f64 = 0.004184;

/// Megajoules in one therm (100,000 British thermal units).
pub const MJ_PER_THERM: f64 = 105.505_59;

/// Megajoules in one million British thermal units.
pub const MJ_PER_MMBTU: f64 = 1_055.055_85;

/// Higher-heating-value megajoules per litre (liquid fuels).
pub const GASOLINE_MJ_PER_L: f64 = 34.2;
pub const DIESEL_MJ_PER_L: f64 = 38.6;
pub const HEATING_OIL_MJ_PER_L: f64 = 38.2;
pub const JET_A_MJ_PER_L: f64 = 35.3;
pub const PROPANE_MJ_PER_L: f64 = 25.3;

/// Approximate higher-heating-value megajoules per barrel of crude oil.
pub const CRUDE_MJ_PER_BARREL: f64 = 6_120.0;

/// One row on the megajoules prices page.
#[derive(Debug, Clone)]
pub struct MegajoulePriceRow {
    pub category: &'static str,
    pub commodity: &'static str,
    pub region: &'static str,
    pub usd_per_mj: f64,
    pub note: String,
}

/// Converts a US dollar per kilowatt-hour price to US dollars per megajoule.
pub fn usd_per_kwh_to_mj(usd_per_kwh: f64) -> f64 {
    usd_per_kwh / MJ_PER_KWH
}

/// Converts a US dollar per litre price to US dollars per megajoule.
pub fn usd_per_liter_to_mj(usd_per_liter: f64, mj_per_liter: f64) -> f64 {
    usd_per_liter / mj_per_liter
}

/// Converts a US dollar per US gallon price to US dollars per megajoule.
pub fn usd_per_gallon_to_mj(usd_per_gallon: f64, mj_per_liter: f64) -> f64 {
    usd_per_liter_to_mj(usd_per_gallon / LITERS_PER_US_GALLON, mj_per_liter)
}

/// Converts a US dollar per kilogram food price using kilocalories per kilogram.
pub fn usd_per_kg_food_to_mj(usd_per_kg: f64, kcal_per_kg: f64) -> f64 {
    usd_per_kg / (kcal_per_kg * KCAL_TO_MJ)
}

fn electricity(
    commodity: &'static str,
    region: &'static str,
    usd_per_kwh: f64,
    note: String,
) -> MegajoulePriceRow {
    MegajoulePriceRow {
        category: "electricity",
        commodity,
        region,
        usd_per_mj: usd_per_kwh_to_mj(usd_per_kwh),
        note,
    }
}

fn fuel_gallon(
    commodity: &'static str,
    region: &'static str,
    usd_per_gallon: f64,
    mj_per_liter: f64,
    note: String,
) -> MegajoulePriceRow {
    MegajoulePriceRow {
        category: "fuel",
        commodity,
        region,
        usd_per_mj: usd_per_gallon_to_mj(usd_per_gallon, mj_per_liter),
        note,
    }
}

fn food_per_kg(
    commodity: &'static str,
    region: &'static str,
    usd_per_kg: f64,
    kcal_per_kg: f64,
    note: String,
) -> MegajoulePriceRow {
    MegajoulePriceRow {
        category: "food",
        commodity,
        region,
        usd_per_mj: usd_per_kg_food_to_mj(usd_per_kg, kcal_per_kg),
        note,
    }
}

fn lb_to_kg(usd_per_lb: f64) -> f64 {
    usd_per_lb / 0.45359237
}

/// Curated October 2026 snapshot (or latest 2026 figure when October is unpublished).
pub fn megajoule_price_rows() -> Vec<MegajoulePriceRow> {
    let mut rows = vec![
        electricity(
            "household electricity",
            "India",
            0.058,
            "from 0.058 USD/kWh, typical urban slab, March 2026 (state tariff compilation)".to_string(),
        ),
        electricity(
            "household electricity",
            "China",
            0.073,
            "from 0.073 USD/kWh, Beijing tiered residential tariff (older schedule; treat as approximate)".to_string(),
        ),
        electricity(
            "household electricity",
            "Brazil",
            0.153,
            "from 0.153 USD/kWh, ANEEL / Enel São Paulo, July 2026".to_string(),
        ),
        electricity(
            "household electricity",
            "Texas",
            0.1594,
            "from 0.1594 USD/kWh, EIA Electric Power Monthly Table 5.6.A, June 2026 residential".to_string(),
        ),
        electricity(
            "household electricity",
            "South Africa",
            0.17,
            "from 0.17 USD/kWh, Eskom standard prices 2026/27, April 2026".to_string(),
        ),
        electricity(
            "household electricity",
            "United States",
            0.1834,
            "from 0.1834 USD/kWh, US residential average, October 2026 (EIA-based compilations)".to_string(),
        ),
        electricity(
            "household electricity",
            "Japan",
            0.229,
            "from 0.229 USD/kWh, TEPCO Energy Partner tariff, May 2026".to_string(),
        ),
        electricity(
            "household electricity",
            "Norway",
            0.243,
            "from 0.243 USD/kWh, Eurostat household band, 2025 second half".to_string(),
        ),
        electricity(
            "household electricity",
            "European Union",
            0.26,
            "from 0.26 USD/kWh, Europe residential regional average, first quarter 2026".to_string(),
        ),
        electricity(
            "household electricity",
            "France",
            0.299,
            "from 0.299 USD/kWh, Eurostat household band, 2025 second half".to_string(),
        ),
        electricity(
            "household electricity",
            "California",
            0.3474,
            "from 0.3474 USD/kWh, EIA Electric Power Monthly Table 5.6.A, June 2026 residential".to_string(),
        ),
        electricity(
            "household electricity",
            "Australia",
            0.349,
            "from 0.349 USD/kWh, AER Default Market Offer 2026–27, July 2026".to_string(),
        ),
        electricity(
            "household electricity",
            "United Kingdom",
            0.359,
            "from 0.359 USD/kWh, Ofgem price-cap unit rate, October 2026".to_string(),
        ),
        electricity(
            "household electricity",
            "Germany",
            0.452,
            "from 0.452 USD/kWh, Eurostat household band, 2025 second half".to_string(),
        ),
        fuel_gallon(
            "gasoline (regular)",
            "United States",
            4.356,
            GASOLINE_MJ_PER_L,
            "from 4.356 USD/US gal, AAA national average, 12 October 2026; 34.2 MJ/L higher heating value"
                .to_string(),
        ),
        fuel_gallon(
            "diesel",
            "United States",
            6.266,
            DIESEL_MJ_PER_L,
            "from 6.266 USD/US gal, US average, 12 October 2026; 38.6 MJ/L higher heating value"
                .to_string(),
        ),
        fuel_gallon(
            "heating oil",
            "United States",
            6.042,
            HEATING_OIL_MJ_PER_L,
            "from 6.042 USD/US gal, EIA SHOPP residential, week of 5 October 2026; 38.2 MJ/L HHV"
                .to_string(),
        ),
        fuel_gallon(
            "propane",
            "United States",
            2.598,
            PROPANE_MJ_PER_L,
            "from 2.598 USD/US gal, EIA SHOPP residential, week of 5 October 2026; 25.3 MJ/L HHV"
                .to_string(),
        ),
        fuel_gallon(
            "jet fuel (Jet A)",
            "US Gulf Coast",
            4.342,
            JET_A_MJ_PER_L,
            "from 4.342 USD/US gal, EIA Gulf Coast spot, 6 October 2026; 35.3 MJ/L HHV".to_string(),
        ),
        MegajoulePriceRow {
            category: "fuel",
            commodity: "natural gas (residential)",
            region: "New York",
            usd_per_mj: 2.987 / MJ_PER_THERM,
            note: "from 2.987 USD/therm, NYSERDA residential monthly average, October 2026"
                .to_string(),
        },
        MegajoulePriceRow {
            category: "fuel",
            commodity: "natural gas (wholesale)",
            region: "United States (Henry Hub)",
            usd_per_mj: 3.12 / MJ_PER_MMBTU,
            note: "from 3.12 USD/MMBtu, Henry Hub spot, 12 October 2026".to_string(),
        },
        MegajoulePriceRow {
            category: "fuel",
            commodity: "crude oil (Brent)",
            region: "International",
            usd_per_mj: 105.91 / CRUDE_MJ_PER_BARREL,
            note: "from 105.91 USD/barrel, 12 October 2026; ~6120 MJ/barrel higher heating value"
                .to_string(),
        },
        MegajoulePriceRow {
            category: "fuel",
            commodity: "thermal coal (Newcastle 6,000 kcal/kg)",
            region: "Asia-Pacific",
            usd_per_mj: 148.85 / (6000.0 * KCAL_TO_MJ * 1000.0),
            note: "from 148.85 USD/metric tonne, Newcastle front-month, 10 October 2026; 6000 kcal/kg net as received"
                .to_string(),
        },
        food_per_kg(
            "white sugar",
            "United States",
            lb_to_kg(1.024),
            3870.0,
            "from 1.024 USD/lb, BLS US city average, July 2026; 3870 kcal/kg".to_string(),
        ),
        food_per_kg(
            "white rice (uncooked)",
            "United States",
            lb_to_kg(1.100),
            3650.0,
            "from 1.100 USD/lb, BLS US city average, July 2026; 3650 kcal/kg".to_string(),
        ),
        food_per_kg(
            "white bread",
            "United States",
            lb_to_kg(1.820),
            2650.0,
            "from 1.820 USD/lb, BLS US city average, July 2026; 2650 kcal/kg".to_string(),
        ),
        food_per_kg(
            "whole chicken",
            "United States",
            lb_to_kg(2.005),
            2190.0,
            "from 2.005 USD/lb, BLS US city average, July 2026; 2190 kcal/kg".to_string(),
        ),
        food_per_kg(
            "ground beef",
            "United States",
            lb_to_kg(6.885),
            2540.0,
            "from 6.885 USD/lb, BLS US city average, July 2026; 2540 kcal/kg".to_string(),
        ),
        food_per_kg(
            "butter",
            "United States",
            lb_to_kg(3.58),
            7170.0,
            "from 3.58 USD/lb advertised, USDA AMS, 26 September–8 October 2026; 7170 kcal/kg"
                .to_string(),
        ),
        food_per_kg(
            "whole milk",
            "United States",
            2.82 / LITERS_PER_US_GALLON,
            610.0,
            "from 2.82 USD/gal advertised, USDA AMS, 26 September–8 October 2026; 610 kcal/kg"
                .to_string(),
        ),
        food_per_kg(
            "large eggs",
            "United States",
            2.189 / 0.6,
            1440.0,
            "from 2.189 USD/dozen, BLS US city average, July 2026; 12 large eggs ≈ 0.6 kg, 1440 kcal/kg"
                .to_string(),
        ),
        food_per_kg(
            "vegetable oil",
            "United States",
            3.69 / (48.0 * 0.029_573_529_562_5 * 0.92),
            8840.0,
            "from 3.69 USD per 48 fl oz, Amazon Grocery vegetable oil, 3 October 2026; 0.92 kg/L, 8840 kcal/kg"
                .to_string(),
        ),
    ];

    rows.sort_by(|a, b| {
        a.category
            .cmp(b.category)
            .then(a.usd_per_mj.partial_cmp(&b.usd_per_mj).unwrap())
    });
    rows
}

/// Formats a US-dollar-per-megajoule value for the table.
pub fn format_usd_per_mj(value: f64) -> String {
    format!("{:.4}", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usd_per_kwh_to_mj_divides_by_3_6() {
        assert!((usd_per_kwh_to_mj(0.36) - 0.1).abs() < 1e-12);
        assert!((usd_per_kwh_to_mj(3.6) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn food_and_fuel_helpers_match_the_documented_formulas() {
        assert!((usd_per_liter_to_mj(34.2, GASOLINE_MJ_PER_L) - 1.0).abs() < 1e-12);
        let from_gallon = usd_per_gallon_to_mj(LITERS_PER_US_GALLON, GASOLINE_MJ_PER_L);
        assert!((from_gallon - usd_per_liter_to_mj(1.0, GASOLINE_MJ_PER_L)).abs() < 1e-12);
        assert!((usd_per_kg_food_to_mj(4.184, 1000.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn snapshot_includes_electricity_fuel_and_food() {
        let rows = megajoule_price_rows();
        assert!(rows.iter().any(|row| row.category == "electricity"));
        assert!(rows.iter().any(|row| row.category == "fuel"));
        assert!(rows.iter().any(|row| row.category == "food"));
        assert!(rows
            .iter()
            .all(|row| row.usd_per_mj.is_finite() && row.usd_per_mj > 0.0));
    }
}
