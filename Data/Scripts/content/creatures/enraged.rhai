// BaseEnraged(summoner): on OSI all stats are random 50-200, but hits never exceed str and stamina never exceeds
// dex (str and dex are rolled by the template). The summoner becomes the master.
fn on_spawn(e, summoner) {
    set_hits_range(e, 50, e.str);
    set_stam_range(e, 50, e.dex);
    set_master(e, summoner);
}
