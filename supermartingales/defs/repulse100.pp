x = 10;
while x >= 0 do
    assert(x <= 100);
    p ~ Bernoulli(0.5);
    if p == 1 then
        x -= 2
    else
        x += 1
    fi
od
