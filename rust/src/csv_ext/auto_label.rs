/*
If there's a header
    Check for a text/data/content/tweet section -- the data col
    Check for a rating/polarity/category/class section -- the rating col
Otherwise, pick x random entries and
    For data col, look for the col with largest averrage size, but with strictly < 280 (or 4000 ?? apparently us subs can ?)
        A little tricky since we need to check for encoding, 
    For rating, look for decimals only, where all are >=0 & <=4.
*/