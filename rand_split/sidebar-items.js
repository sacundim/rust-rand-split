initSidebarItems({"macro":[["split_rand_seq_impl!","A macro that implements `SplitRand` sequentially for any type that has a `Rand` implementation, simply by using that.  This is meant to be used for \"atomic\" types whose generation doesn't benefit from splittability."]],"mod":[["chaskeyrng","A splittable pseudorandom generator based on the Chaskey MAC.  **This is not intended to be a cryptographically secure PRNG.**"],["generic","A construction that turns a pair of a splittable and a sequential PRNG into a splittable PRNG.  The intent of this is to gain the splittability of the former but retain the sequential generation speed of the latter."],["siprng","A splittable pseudo-random number generator based on the SipHash function.  **This is not intended to be a cryptographically secure PRNG.**"],["twolcg","A splittable pseudorandom generator based on the TwoLCG algorithm."]],"struct":[["Seq","A newtype wrapper to add a `SplitRand` implementation to `Rand` types.  This just does the same thing as the base type's `Rand` one does."]],"trait":[["SplitPrf","Pseudo-random functions (\"PRFs\") generated off a `SplitRng`."],["SplitRand","A type that can be randomly generated from a `SplitRand`. Implementations are expected to exploit splittability where possible."],["SplitRng","A trait for **splittable** pseudo random generators."]],"type":[["Prf","The pseudo-random functions of a generic `Split` RNG."],["Split","A wrapper that generically adds splittability to RNGs."]]});