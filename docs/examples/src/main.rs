mod blowfish_99;
mod deer_mushroom_zombie_fly;
mod mushroom_scorpion;
mod permanent_coconut;
mod pet_combinations;
mod rhino;

use blowfish_99::ninety_nine_blowfish_battle;
use deer_mushroom_zombie_fly::deer_fly_mushroom;
use mushroom_scorpion::mushroom_scorpion;
use permanent_coconut::permanent_coconut;
use pet_combinations::five_pet_combinations;
use rhino::rhino;

fn main() {
    // Rhino ability.
    let _rhino_team = rhino();
    // Food abilities and summon order.
    let _deer_team = deer_fly_mushroom();
    // Expanded team sizes.
    let _blowfish_team = ninety_nine_blowfish_battle();
    // Shops and turn orders.
    let _permanent_coconut_team = permanent_coconut();
    // Battle mechanics
    let _scorpion_team = mushroom_scorpion();

    // Generate all 5-pet combinations from the Turtle pack.
    let _teams = five_pet_combinations();
}
