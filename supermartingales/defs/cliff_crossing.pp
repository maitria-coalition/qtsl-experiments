x = 0;
y = 0;
while x <= 50 do
    assert(y <= 10);
    p ~ Bernoulli(0.9);
    x += 1;
    if p == 0 then
        y += 1
    else
        if y >= 1 then
            y -= 1
        fi
    fi
od
