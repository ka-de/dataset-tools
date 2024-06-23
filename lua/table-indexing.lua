-- Create a table with 5 quotes from Terence McKenna
quotes = {
    "Nature is not our enemy, to be raped and conquered. Nature is ourselves, to be cherished and explored.",
    "The cost of sanity in this society, is a certain level of alienation",
    "We have to create culture, don't watch TV, don't read magazines, don't even listen to NPR. Create your own roadshow.",
    "You are a divine being. You matter, you count. You come from realms of unimaginable power and light, and you will return to those realms.",
    "The syntactical nature of reality, the real secret of magic, is that the world is made of words. And if you know the words that the world is made of, you can make of it whatever you wish."
}

-- Delete the second item
table.remove(quotes, 2)

-- Print out the length of the array
print("Length of the array: " .. #quotes)

-- Print out all the quotes
for i, quote in ipairs(quotes) do
    print("Quote " .. i .. ": " .. quote)
end
