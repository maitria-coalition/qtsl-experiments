x = ?;  # init unspecified in intro figure; A = {x > 10}
while x > 0 do
    assert(x <= 10);
    p ~ Bernoulli(0.5);
    if p == 1 then
        x -= 2
    else
        x += 1
    fi
od
